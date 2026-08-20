// SPDX-License-Identifier: GPL-3.0-or-later

//We need to expose some grid interface to cfd-assemble
//TODO: refractor model to put cfd oriented into separated mod and move "reactor model" such as 0d/pfr from assemble to here

use crate::{
    CoreError,
    coordinates::{CartesianCoordinates, CartesianVec3},
    errors::ModelError,
    model::{
        compartments::{CompartmentInfo, CountVolumeElement, ElementVolumeInfo},
        interfaces::{AInterfacesInfo, InterfaceFlow, InterfaceInfo},
    },
};
use std::{f64, sync::Arc};
mod data;
use cmtool_data::{FluxFileHeader, RawDataFlux, RawDataScalar, RawFlux};
mod balance;
mod compartments;
mod geometry;
mod interfaces;
mod scalar;
mod vectors;
pub use scalar::Scalar;
pub use vectors::Vector as ModelVector;
pub use vectors::Vector;

pub use balance::{BalanceReport, BalanceSettings};

pub struct CMModel {
    geometry: Arc<CMGeometry>,
    c_info: CompartmentInfo,
    interfaces: AInterfacesInfo,
}

pub use geometry::CMGeometry;

fn get_data_flow(
    n_zones: usize,
    i_info: &[InterfaceInfo],
    i_flow: &[InterfaceFlow],
) -> RawDataFlux {
    let fluxes: Vec<RawFlux> = i_info
        .iter()
        .zip(i_flow)
        .map(|(k_i_nfo, k_i_flow)| RawFlux {
            id_source: k_i_nfo.source_id as u32,
            id_target: k_i_nfo.target_id as u32,
            flux_source_target: k_i_flow.source_flow,
            flux_target_source: k_i_flow.target_flow,
        })
        .collect();
    RawDataFlux {
        header: FluxFileHeader {
            n_fluxes: i_flow.len() as u32,
            n_zone: n_zones as u32,
        },
        fluxes,
    }
}

impl CMModel {
    ///Both directions must carry a usable flow and both ids must address a compartment,
    ///check_flow indexes mass_balance with them right after
    fn check_flux(n_zone: u32, rf: &RawFlux) -> bool {
        let is_flow_valid = |flow: f64| flow.is_finite() && flow.is_sign_positive();

        is_flow_valid(rf.flux_source_target)
            && is_flow_valid(rf.flux_target_source)
            && rf.id_source < n_zone
            && rf.id_target < n_zone
    }

    pub fn check_flow(&self, raw: &RawDataFlux, max_divergence: f64) -> Result<(), ModelError> {
        const ABS_TOLERANCE_DIVERGENCE_CELL: f64 = 1e-7;

        //A field of zeros balances perfectly and transports nothing: it is missing data, not a
        //valid flow map
        if raw
            .fluxes
            .iter()
            .all(|flux| flux.flux_source_target == 0. && flux.flux_target_source == 0.)
        {
            return Err(ModelError::EmptyFlow);
        }

        let mut mass_balance: Vec<InterfaceFlow> =
            vec![InterfaceFlow::default(); raw.header.n_zone as usize];
        let mut id_max = u32::MIN;

        for flow in raw.fluxes.iter() {
            if !Self::check_flux(raw.header.n_zone, flow) {
                return Err(ModelError::InvalidFlow);
            }

            id_max = u32::max(id_max, flow.id_source);
            //out
            mass_balance[flow.id_source as usize].source_flow += flow.flux_target_source;
            //in
            mass_balance[flow.id_source as usize].target_flow += flow.flux_source_target;

            //in
            mass_balance[flow.id_target as usize].source_flow += flow.flux_source_target;
            //out
            mass_balance[flow.id_target as usize].target_flow += flow.flux_target_source;
        }

        for flow in &mass_balance {
            let abs_diff = (flow.source_flow - flow.target_flow).abs();
            let denom = flow.source_flow.abs() + flow.target_flow.abs();

            let err = if denom > f64::EPSILON {
                2.0 * abs_diff / denom
            } else {
                abs_diff
            };
            if abs_diff > ABS_TOLERANCE_DIVERGENCE_CELL && err > max_divergence {
                return Err(ModelError::CellToCellDivergence(err, max_divergence));
            }
        }

        Ok(())
    }
}

impl CMModel {
    pub fn grid(&self) -> &dyn crate::grid::CompartmentMesh {
        self.geometry.get_grid().unwrap()
    }

    pub fn init(geometry: Arc<CMGeometry>) -> Self {
        println!("Init model with {} compartment", geometry.n_zone());
        let volume_element_count = geometry.get_count_volume_element_first_pass();

        let n_zone_with_volume_element = volume_element_count.n_zone_with_volume_element();

        if n_zone_with_volume_element != geometry.n_zone() {
            unimplemented!(
                "Detected compartment should be the same as given by user {} vs {}",
                n_zone_with_volume_element,
                geometry.n_zone()
            )
        }

        let interface_count_raw = volume_element_count.at_interface.clone();

        let (c_info, interfaces) = volume_element_count.into_reduce();

        // if interfaces.n_interfaces() >= geometry.get_grid().as_ref().unwrap().n_maximum_interface()
        if interfaces.n_interfaces() > geometry.get_grid().as_ref().unwrap().n_maximum_interface() {
            unimplemented!(
                "RCMTOOL: CMModel::init: should have intefaces  < n_maximum_interface {} {}",
                interfaces.n_interfaces(),
                geometry.get_grid().as_ref().unwrap().n_maximum_interface()
            )
        }

        let mut model = Self {
            geometry,
            c_info,
            interfaces,
        };

        model.c_info.fill(&model.geometry);

        model.interfaces.fill(&model.geometry, &interface_count_raw);

        model
    }

    #[allow(unused)]
    fn compute_volume_integral_per_zone() -> Vec<f64> {
        todo!()
    }

    ///Velocity of one element along the normal of the interface it crosses.
    ///
    ///The velocity stays a vector up to here and is projected on the *local* normal: on a radial
    ///or theta face the normal turns with theta, so it has to be taken at the position of the
    ///element and not once for the whole interface.
    fn normal_velocity(&self, vector: &Vector, element_global_id: usize, axis: usize) -> f64 {
        let velocity = CartesianVec3(*vector.get_slice_xyz(element_global_id));
        let CartesianCoordinates(centroid) = self.geometry.volume_elements.xyz[element_global_id];
        let theta = centroid[1].atan2(centroid[0]);

        velocity.to_cylindrical_vec(theta).0[axis]
    }

    pub fn compute_flux_between_compartments(
        &self,
        vector: Vector,
        settings: &BalanceSettings,
    ) -> Result<cmtool_data::RawDataFlux, CoreError> {
        let n_fluxes = self.interfaces.n_interfaces();
        let mut flows: Vec<InterfaceFlow> = vec![Default::default(); n_fluxes];

        //Flux of an interface is the surface integral of the velocity over the area the elements
        //really cover, element by element: sum(a_e * v_e.n_e). Taking an average velocity times
        //the geometric face instead would count area the vessel does not have, which is what
        //inflates the flow of the compartments sitting on its boundary.
        //
        //Both directions are accumulated separately, so a counter current inside one interface is
        //kept as gross exchange and neither direction can come out negative.
        for (i_interface, flow) in flows.iter_mut().enumerate() {
            let axis: usize = self.interfaces.normal_axis[i_interface];
            let areas = &self.interfaces.area[i_interface];
            let elements = &self.interfaces.global_id_from_interface[i_interface];

            for (&element_global_id, area) in elements.iter().zip(areas) {
                let f = area * self.normal_velocity(&vector, element_global_id, axis);

                //The normal of the interface points from its source to its target
                if f > 0. {
                    flow.source_flow += f;
                } else {
                    flow.target_flow -= f;
                }
            }
        }

        let balance = self.balance(&mut flows, settings);
        if balance.residual > settings.max_divergence {
            return Err(ModelError::CellToCellDivergence(
                balance.residual,
                settings.max_divergence,
            )
            .into());
        }

        let data_flow = get_data_flow(self.geometry.n_zone(), &self.interfaces.ids, &flows);

        self.check_flow(&data_flow, settings.max_divergence)?;
        Ok(data_flow)
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    ///Balance flow
    ///
    ///A flow is only ever multiplied by a positive factor. Scaling the
    ///flows leaving a compartment by `sqrt(in / out)` moves it halfway to its balance, and
    ///repeating it converges the same way Sinkhorn balancing does.
    fn balance(&self, flows: &mut [InterfaceFlow], settings: &BalanceSettings) -> BalanceReport {
        let n_zone = self.geometry.n_zone();
        let mut inflow = vec![0.0f64; n_zone];
        let mut outflow = vec![0.0f64; n_zone];
        let mut report = BalanceReport {
            iterations: 0,
            residual: f64::INFINITY,
        };

        for iteration in 0..settings.max_iterations {
            inflow.fill(0.);
            outflow.fill(0.);
            for (i_interface, flow) in flows.iter().enumerate() {
                let source_id = self.interfaces.ids[i_interface].source_id;
                let target_id = self.interfaces.ids[i_interface].target_id;
                outflow[source_id] += flow.source_flow;
                inflow[target_id] += flow.source_flow;
                outflow[target_id] += flow.target_flow;
                inflow[source_id] += flow.target_flow;
            }

            report.iterations = iteration;
            report.residual = (0..n_zone)
                .map(|zone| {
                    let total = inflow[zone].abs() + outflow[zone].abs();
                    if total > f64::EPSILON {
                        2. * (inflow[zone] - outflow[zone]).abs() / total
                    } else {
                        0.
                    }
                })
                .fold(0.0f64, f64::max);

            if report.residual < settings.tolerance {
                break;
            }

            //A compartment with no flow either way has nothing to scale
            let factor: Vec<f64> = (0..n_zone)
                .map(|zone| {
                    if inflow[zone] > f64::EPSILON && outflow[zone] > f64::EPSILON {
                        (inflow[zone] / outflow[zone]).sqrt()
                    } else {
                        1.
                    }
                })
                .collect();

            for (i_interface, flow) in flows.iter_mut().enumerate() {
                let source_id = self.interfaces.ids[i_interface].source_id;
                let target_id = self.interfaces.ids[i_interface].target_id;
                flow.source_flow *= factor[source_id];
                flow.target_flow *= factor[target_id];
            }
        }

        report
    }

    pub fn export_volume_integral_per_zone(
        &self,
        model_scalar: Scalar,
    ) -> Result<cmtool_data::RawDataScalar, CoreError> {
        println!("Creating scalar {}", model_scalar.name.trim(),);
        let mut scalar_field = RawDataScalar::new(self.geometry.n_zone());

        scalar_field.values = vec![(0.).into(); self.geometry.n_zone()];

        let iterator = self.c_info.volumes.iter().zip(&mut scalar_field.values);

        for (volumes_i, field) in iterator {
            field.value =
                volumes_i
                    .iter()
                    .fold(0.0, |acc, ElementVolumeInfo { global_id, volume }| {
                        acc + model_scalar[*global_id] * volume
                    });
        }

        Ok(scalar_field)
    }

    pub fn compartments_volumes(&self) -> Vec<f64> {
        todo!("grid compartment calculation")
    }

    pub fn get_real_volume(&self) -> Vec<f64> {
        self.c_info
            .volumes
            .iter()
            .map(|zone| zone.iter().map(|v| v.volume).sum())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const N_ZONE: u32 = 2;

    fn flux(id_source: u32, id_target: u32, source_target: f64, target_source: f64) -> RawFlux {
        RawFlux {
            id_source,
            id_target,
            flux_source_target: source_target,
            flux_target_source: target_source,
        }
    }

    #[test]
    fn test_check_flux_accepts_valid_flux() {
        assert!(CMModel::check_flux(N_ZONE, &flux(0, 1, 2., 0.)));
    }

    ///An id out of range would index mass_balance out of bounds in check_flow
    #[test]
    fn test_check_flux_rejects_unknown_compartment() {
        assert!(!CMModel::check_flux(N_ZONE, &flux(N_ZONE, 1, 2., 0.)));
        assert!(!CMModel::check_flux(N_ZONE, &flux(0, N_ZONE, 2., 0.)));
    }

    #[test]
    fn test_check_flux_rejects_unusable_flow() {
        assert!(!CMModel::check_flux(N_ZONE, &flux(0, 1, f64::NAN, 0.)));
        assert!(!CMModel::check_flux(N_ZONE, &flux(0, 1, -2., 0.)));
        //Both directions are checked, not only source to target
        assert!(!CMModel::check_flux(N_ZONE, &flux(0, 1, 2., f64::INFINITY)));
        assert!(!CMModel::check_flux(N_ZONE, &flux(0, 1, 2., -1.)));
    }
}
