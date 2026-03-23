// SPDX-License-Identifier: GPL-3.0-or-later

//We need to expose some grid interface to cfd-assemble
//TODO: refractor model to put cfd oriented into separated mod and move "reactor model" such as 0d/pfr from assemble to here

use crate::{
    CoreError,
    coordinates::{CartesianCoordinates, CartesianVec3, CylindricalCoordinates},
    grid::MeshType,
    model::{
        compartments::{CompartmentInfo, CountVolumeElement, ElementVolumeInfo},
        interfaces::{AInterfacesInfo, InterfaceFlow, InterfaceInfo},
    },
};
use std::sync::Arc;
mod data;
use cmtool_data::{RawDataFlux, RawDataScalar};
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

        if interfaces.n_interfaces() >= geometry.get_grid().as_ref().unwrap().n_maximum_interface()
        {
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

    pub fn compute_flux_between_compartments(
        &self,
        vector: Vector,
    ) -> Result<cmtool_data::RawDataFlux, CoreError> {
        let n_fluxes = self.interfaces.n_interfaces();
        let mut flux_field = RawDataFlux::new(self.geometry.n_zone(), n_fluxes);

        let mut flows: Vec<InterfaceFlow> = vec![Default::default(); n_fluxes];

        // for (i_interface, flow) in flows.iter_mut().enumerate() {
        //     let axis: usize = self.interfaces.normal_axis[i_interface];
        //     let current_interface_area = &self.interfaces.area[i_interface];
        //     let curent_inteface_element = &self.interfaces.global_id_from_interface[i_interface];

        //     for (global_id, area) in curent_inteface_element.iter().zip(current_interface_area) {
        //         let vector_value = CartesianVec3(vector.get_slice_xyz(*global_id).to_owned());

        //         let coords = match self.geometry.mesh_type {
        //             MeshType::Cylindrical => {
        //                 let CartesianCoordinates(centroid) =
        //                     self.geometry.volume_elements.xyz[*global_id];

        //                 let CylindricalCoordinates(centroid) =
        //                     CartesianCoordinates(centroid).into();

        //                 vector_value.to_cylindrical_vec(centroid[1]).0

        //                 // let cyl_vec = vector_value
        //                 //     .to_cylindrical_vec(self.interfaces.interface_theta[i_interface])
        //                 //     .0;
        //                 // cyl_vec
        //                 //
        //                 //
        //                 //
        //                 // let r = (centroid[0].powi(2) + centroid[1].powi(2)).sqrt();
        //                 // match axis {
        //                 //     1 => [cyl_vec[0], cyl_vec[1] * r, cyl_vec[2]],
        //                 //     _ => cyl_vec,
        //                 // }
        //             }
        //             _ => vector_value.0,
        //         };

        //         let f = coords[axis] * area;

        //         if f > 0. {
        //             flow.source_flow += f
        //         } else if f < 0. {
        //             flow.target_flow += f.abs()
        //         }
        //     }
        // }

        for (i_interface, flow) in flows.iter_mut().enumerate() {
            let axis: usize = self.interfaces.normal_axis[i_interface];
            let current_interface_area = &self.interfaces.area[i_interface];
            let curent_inteface_element = &self.interfaces.global_id_from_interface[i_interface];

            let total_area: f64 = current_interface_area.iter().sum();
            let v_avg = curent_inteface_element
                .iter()
                .zip(current_interface_area)
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
                });

            let coords = v_avg;

            let source_id = self.interfaces.ids[i_interface].source_id;
            let axis_oriented = crate::grid::index_to_oriented(axis);
            let theoretical_area = self
                .geometry
                .get_grid()
                .unwrap()
                .cell_surface(source_id, axis_oriented);

            let f = coords[axis] * theoretical_area;

            if f > 0. {
                flow.source_flow += f
            } else if f < 0. {
                flow.target_flow += f.abs()
            }
        }

        self.clean(&mut flows);

        for (i_interface, rd) in flux_field.fluxes.iter_mut().enumerate() {
            let InterfaceInfo {
                source_id,
                target_id,
            } = self.interfaces.ids[i_interface];
            rd.id_source = source_id as u32;
            rd.id_target = target_id as u32;
            let InterfaceFlow {
                source_flow,
                target_flow,
            } = &flows[i_interface];
            rd.flux_source_target = *source_flow;
            rd.flux_target_source = *target_flow;
        }

        // for (i_interface, rd) in flux_field.fluxes.iter_mut().enumerate() {
        //     let InterfaceInfo {
        //         source_id,
        //         target_id,
        //     } = self.interfaces.ids[i_interface];

        //     rd.id_source = source_id as u32;
        //     rd.id_target = target_id as u32;

        //     let InterfaceFlow {
        //         source_flow,
        //         target_flow,
        //     } = &flows[i_interface];
        //     rd.flux_source_target = *source_flow;
        //     rd.flux_target_source = *target_flow;
        // }

        Ok(flux_field)
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    fn clean(&self, flows: &mut [InterfaceFlow]) {
        const TOL: f64 = 1e-19;
        const TOL_CONV: f64 = 1e-12;
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
            let total_balance = _balance.iter().map(|&x| x.abs()).sum::<f64>();
            let total_flow = _flows
                .iter()
                .map(|flow| flow.target_flow + flow.source_flow)
                .sum::<f64>();

            total_balance / total_flow
        };

        let mut conv = false;
        let mut it = 0;
        let mut err = 0.;

        while it < MAX_IT && !conv {
            balance.fill(0.0);
            let max_err = f_err(&mut balance, flows);
            if max_err < TOL {
                break;
            }
            println!("{} {}", max_err, it);
            conv = (err - max_err).abs() < TOL_CONV;
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

        // for i in 0..MAX_IT {
        //     balance.fill(0.0);

        //     let max_err = f_err(&mut balance, flows);

        //     println!("{} {}", max_err, i);
        //     if max_err < TOL {
        //         break;
        //     }
        //     for (i_interface, flow) in flows.iter_mut().enumerate() {
        //         let source_id = self.interfaces.ids[i_interface].source_id;
        //         let target_id = self.interfaces.ids[i_interface].target_id;
        //         let corr = (balance[source_id] - balance[target_id])
        //             / (n_interfaces_per_zone[source_id] + n_interfaces_per_zone[target_id]) as f64;
        //         flow.source_flow += corr;
        //         flow.target_flow -= corr;
        //     }
        // }
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
