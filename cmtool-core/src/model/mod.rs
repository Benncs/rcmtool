use crate::{
    CoreError,
    coordinates::CartesianCoordinates,
    ensight_gold::types::VolumeElementTypes,
    grid::MeshType,
    model::{
        compartments::{CompartmentInfo, CountVolumeElement, ElementVolumeInfo},
        interfaces::{AInterfacesInfo, InterfaceFlow, InterfaceInfo},
    },
    utils::{self, compute_volume},
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
        // let n_interfaces = volume_element_count.n_interfaces();

        let (c_info, interfaces) = volume_element_count.into_reduce();
        
        let n_max_interface = geometry.get_grid().as_ref().unwrap().n_maximum_interface();
        if interfaces.n_facet.len()!= n_max_interface
        {
            eprintln!("Intefaces should be n_maximum_interface {} {}",interfaces.n_facet.len(),n_max_interface);
         //   unimplemented!("Intefaces should be n_maximum_interface")
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

    fn compute_flux_through_limits() -> Vec<f64> {
        todo!()
    }

    fn compute_volume_integral_per_zone() -> Vec<f64> {
        todo!()
    }

    pub fn export_flux_through_limits(
        &self,
        vector: Vector,
    ) -> Result<cmtool_data::RawDataFlux, CoreError> {
        let n_fluxes = self.interfaces.n_facet.len();
        let mut flux_field = RawDataFlux::new(self.geometry.n_zone(), n_fluxes);

        let mut flows: Vec<InterfaceFlow> = vec![Default::default(); n_fluxes];

        for (i_interface, flow) in flows.iter_mut().enumerate() {
            let axis: usize = self.interfaces.axis[i_interface];
            let current_interface_area = &self.interfaces.area[i_interface];
            let curent_inteface_element = &self.interfaces.global_id_from_interface[i_interface];

            for (global_id, area) in curent_inteface_element.iter().zip(current_interface_area) {
                let vector_coords = vector.get_slice_xyz(*global_id);
           
                let coords = if self.geometry.mesh_type == MeshType::Cylindrical {
                    let CartesianCoordinates(centroid) =
                        self.geometry.volume_elements.xyz[*global_id];

                    utils::vector_cartesian_to_cylindrical(vector_coords, centroid)
                } else {
                    vector_coords.to_owned()
                };

                let f = coords[axis] * area;

                if f > 0. {
                    flow.source_flow += f
                } else if f < 0. {
                    flow.target_flow += f.abs()
                }
            }
        }

        for (i_interface, rd) in flux_field.fluxes.iter_mut().enumerate() {
            let InterfaceInfo {
                source_id,
                target_id,
            } = self.interfaces.info[i_interface];

            rd.id_source = source_id as u32;
            rd.id_target = target_id as u32;

            let InterfaceFlow {
                source_flow,
                target_flow,
            } = &flows[i_interface];
            rd.flux_source_target = *source_flow;
            rd.flux_target_source = *target_flow;
        }

        Ok(flux_field)
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    pub fn export_volume_integral_per_zone(
        &self,
        scalar: Scalar,
    ) -> Result<cmtool_data::RawDataScalar, CoreError> {
        println!("Creating scalar {}", scalar.name.trim(),);
        let mut scalar_field = RawDataScalar::new(self.geometry.n_zone());

        scalar_field.values = vec![(0.).into(); self.geometry.n_zone()];

        let iterator = self.c_info.volumes.iter().zip(&mut scalar_field.values);

        for (volumes_i, field) in iterator {
            field.value = volumes_i
                .iter()
                .fold(0.0, |acc, ElementVolumeInfo { global_id, volume }| {
                    acc + scalar[*global_id] * volume
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
