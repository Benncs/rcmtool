// SPDX-License-Identifier: GPL-3.0-or-later

//We need to expose some grid interface to cfd-assemble
//TODO: refractor model to put cfd oriented into separated mod and move "reactor model" such as 0d/pfr from assemble to here

use crate::{
    CoreError,
    coordinates::{CartesianCoordinates, CartesianVec3, Coords3},
    errors::ModelError,
    model::{
        compartments::{CompartmentInfo, CountVolumeElement, ElementVolumeInfo},
        interfaces::{AInterfacesInfo, InterfaceFlow, InterfaceInfo},
    },
};
use std::{f64, sync::Arc};
mod data;
use cmtool_data::{FluxFileHeader, RawDataFlux, RawDataScalar, RawFlux};
mod compartments;
mod geometry;
mod interfaces;
mod scalar;
mod vectors;
pub use scalar::Scalar;
pub use vectors::Vector;

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

    // let mut flux_field = RawDataFlux::new(n_zones, i_flow.len());
    // for (i_interface, rd) in flux_field.fluxes.iter_mut().enumerate() {
    //     let InterfaceInfo {
    //         source_id,
    //         target_id,
    //     } = i_info[i_interface];
    //     rd.id_source = source_id as u32;
    //     rd.id_target = target_id as u32;
    //     let InterfaceFlow {
    //         source_flow,
    //         target_flow,
    //     } = &i_flow[i_interface];
    //     rd.flux_source_target = *source_flow;
    //     rd.flux_target_source = *target_flow;
    // }

    // flux_field
}

impl CMModel {
    fn check_flux(n_zone: u32, rf: &RawFlux) -> bool {
        let mut flag = false;
        flag |= rf.flux_source_target.is_finite();
        flag |= rf.flux_source_target.is_sign_positive();
        flag |= rf.id_source < n_zone;
        flag |= rf.id_target < n_zone;
        flag
    }

    pub fn check_flow(&self, raw: &RawDataFlux) -> Result<(), ModelError> {
        const REL_TOLERANCE_DIVERGENCE_CELL: f64 = 1e-2;
        const ABS_TOLERANCE_DIVERGENCE_CELL: f64 = 1e-7;

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
            if abs_diff > ABS_TOLERANCE_DIVERGENCE_CELL && err > REL_TOLERANCE_DIVERGENCE_CELL {
                return Err(ModelError::CellToCellDivergence(
                    err,
                    REL_TOLERANCE_DIVERGENCE_CELL,
                ));
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

    fn get_average_velocity(
        &self,
        vector: &Vector,
        _curent_inteface_element: &[usize],
        __curent_inteface_area: &[f64],
    ) -> Coords3 {
        let total_area: f64 = __curent_inteface_area.iter().sum();
        _curent_inteface_element
            .iter()
            .zip(__curent_inteface_area)
            .fold([0.0f64; 3], |acc, (gid, a)| {
                let v = CartesianVec3(vector.get_slice_xyz(*gid).to_owned());
                let CartesianCoordinates(centroid) = self.geometry.volume_elements.xyz[*gid];
                let theta = centroid[1].atan2(centroid[0]);
                let cyl = v.to_cylindrical_vec(theta).0;
                [
                    acc[0] + cyl[0] * a / total_area,
                    acc[1] + cyl[1] * a / total_area,
                    acc[2] + cyl[2] * a / total_area,
                ]
            })
    }

    pub fn compute_flux_between_compartments(
        &self,
        vector: Vector,
    ) -> Result<cmtool_data::RawDataFlux, CoreError> {
        let n_fluxes = self.interfaces.n_interfaces();
        let mut flows: Vec<InterfaceFlow> = vec![Default::default(); n_fluxes];

        for (i_interface, flow) in flows.iter_mut().enumerate() {
            let axis: usize = self.interfaces.normal_axis[i_interface];

            let axis_oriented = crate::grid::index_to_oriented(axis);

            let source_id = self.interfaces.ids[i_interface].source_id;
            let theoretical_area = self
                .geometry
                .get_grid()
                .unwrap()
                .cell_surface(source_id, axis_oriented);

            let current_interface_area = &self.interfaces.area[i_interface];
            let curent_inteface_element = &self.interfaces.global_id_from_interface[i_interface];
            let v_avg =
                self.get_average_velocity(&vector, curent_inteface_element, current_interface_area);

            let f = v_avg[axis] * theoretical_area;

            if f > 0. {
                flow.source_flow += f
            } else if f < 0. {
                flow.target_flow += f.abs()
            }
        }

        self.clean(&mut flows);

        let data_flow = get_data_flow(self.geometry.n_zone(), &self.interfaces.ids, &flows);

        self.check_flow(&data_flow)?;
        Ok(data_flow)
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    fn clean(&self, flows: &mut [InterfaceFlow]) {
        // const TOL: f64 = 1e-19;
        const ABS_TOL_CONV: f64 = 1e-12;
        const REL_TOL_CONV: f64 = 1e-6;

        const MAX_IT: usize = 10000;
        let mut balance = vec![0.0f64; self.geometry.n_zone()];
        let n_interfaces_per_zone = {
            let mut tmp = vec![0usize; self.geometry.n_zone()];
            for i_interface in 0..flows.len() {
                let source_id = self.interfaces.ids[i_interface].source_id;
                let target_id = self.interfaces.ids[i_interface].target_id;
                tmp[source_id] += 1;
                tmp[target_id] += 1;
            }
            tmp
        };

        // let f_err = |_balance: &mut [f64], _flows: &[InterfaceFlow]| {
        //     for (i_interface, flow) in _flows.iter().enumerate() {
        //         let source_id = self.interfaces.ids[i_interface].source_id;
        //         let target_id = self.interfaces.ids[i_interface].target_id;
        //         _balance[source_id] += flow.target_flow - flow.source_flow;
        //         _balance[target_id] += flow.source_flow - flow.target_flow;
        //     }
        //     _balance.iter().map(|x| x.abs()).fold(0.0f64, f64::max)
        // };

        let f_err = |_balance: &mut [f64], _flows: &[InterfaceFlow]| {
            for (i_interface, flow) in _flows.iter().enumerate() {
                let source_id = self.interfaces.ids[i_interface].source_id;
                let target_id = self.interfaces.ids[i_interface].target_id;
                _balance[source_id] += flow.target_flow - flow.source_flow;
                _balance[target_id] += flow.source_flow - flow.target_flow;
            }
            // let total_balance = _balance.iter().map(|&x| x.abs()).sum::<f64>();
            let total_balance = _balance.iter().map(|&x| x * x).sum::<f64>().sqrt();
            let total_flow = _flows
                .iter()
                .map(|flow| flow.target_flow + flow.source_flow)
                .sum::<f64>();

            if total_flow > f64::EPSILON {
                total_balance / total_flow
            } else {
                total_balance
            }
        };

        let mut conv = false;
        let mut it = 0;
        let mut err = 0.;

        while it < MAX_IT && !conv {
            balance.fill(0.0);
            let max_err = f_err(&mut balance, flows);

            let delta = (err - max_err).abs();

            conv = delta < ABS_TOL_CONV || delta / err.max(f64::EPSILON) < REL_TOL_CONV;
            if conv {
                break;
            }
            err = max_err;
            it += 1;
            for (i_interface, flow) in flows.iter_mut().enumerate() {
                let source_id = self.interfaces.ids[i_interface].source_id;
                let target_id = self.interfaces.ids[i_interface].target_id;
                let corr = (balance[source_id] - balance[target_id])
                    / (n_interfaces_per_zone[source_id] + n_interfaces_per_zone[target_id]) as f64;
                flow.source_flow += corr;
                flow.target_flow -= corr;
            }
        }
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
