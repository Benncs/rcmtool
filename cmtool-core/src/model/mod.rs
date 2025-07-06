use crate::{
    ensight_gold::types::VolumeElementTypes,
    model::{
        compartments::{CompartmentInfo, CountVolumeElement, ElementVolumeInfo, InterfacesInfo},
        scalar::Scalar,
    },
    utils::compute_volume,
    CoreError,
};
use std::sync::Arc;
mod data;
use cmtool_data::RawDataScalar;
mod compartments;
mod geometry;
pub mod scalar;
pub struct CMModel {
    geometry: Arc<CMGeometry>,
    c_info: CompartmentInfo,
    interfaces: InterfacesInfo,
}

pub use geometry::CMGeometry;

impl CMModel {
    pub fn init(geometry: Arc<CMGeometry>) -> Self {
        println!("Init model with {} compartment", geometry.n_zone());
        let volume_element_count = geometry.get_count_volume_element();

        let n_zone_with_volume_element = volume_element_count.n_zone_with_volume_element();

        if n_zone_with_volume_element != geometry.n_zone() {
            unimplemented!(
                "Detected compartment should be the same as given by user {} vs {}",
                n_zone_with_volume_element,
                geometry.n_zone()
            )
        }

        // let n_interfaces = volume_element_count.n_interfaces();

        let (c_info, interfaces) = volume_element_count.into_reduce();
        let mut model = Self {
            geometry,
            c_info,
            interfaces,
        };

        model.fill();

        model
    }

    fn fill(&mut self) {
        let mut v_tot = 0.;
        let mut local_vertices = Vec::new();
        for volume_element_global_id in 0..self.geometry.volume_elements.n_element() {
            let ncid = self
                .geometry
                .volume_elements
                .get_number_cid(volume_element_global_id);
            let n_vertex = self
                .geometry
                .volume_elements
                .get_vertex_per_element(volume_element_global_id);
            let vtype: VolumeElementTypes =
                self.geometry.volume_elements.vtype[volume_element_global_id];
            local_vertices.clear();
            local_vertices.resize(n_vertex, Default::default());
            for k_c in 0..ncid {
                // let tmp = self.geometry
                //     .volume_elements
                //     .get_limit_cell_id(volume_element_global_id, k_c);

                for k_vertex in 0..n_vertex {
                    let vertex_global_id = self
                        .geometry
                        .volume_elements
                        .get_vertex_from_vol_global_id(volume_element_global_id, k_vertex);

                    local_vertices[k_vertex] = self
                        .geometry
                        .vertices
                        .get_slice_xyz(vertex_global_id)
                        .to_owned();
                }
                let volume = compute_volume(&local_vertices, vtype).unwrap() / (ncid as f64);
                v_tot += volume;
            }
        }
        println!("{}", v_tot);
    }

    fn compute_flux_through_limits() -> Vec<f64> {
        todo!()
    }

    fn compute_volume_integral_per_zone() -> Vec<f64> {
        todo!()
    }

    pub fn export_flux_through_limits(&self, flow: &mut cmtool_data::RawDataFlux) {
        todo!()
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    pub fn export_volume_integral_per_zone(
        &self,
        scalar: Scalar,
    ) -> Result<cmtool_data::RawDataScalar, CoreError> {
        println!(
            "Creating scalar with {} compartment",
            self.geometry.n_zone()
        );
        let mut scalar_field = RawDataScalar::new(self.geometry.n_zone());

        for (compartment_id, n_volume_element) in self.c_info.n_volumes.iter().enumerate() {
            scalar_field.values.push((0.).into());
            for k_volume_element in 0..*n_volume_element {
                let ElementVolumeInfo { global_id, volume } =
                    self.c_info.volumes[compartment_id][k_volume_element];
                let value = scalar[global_id];
                scalar_field.values[compartment_id].value += value * volume;
            }
        }

        todo!("export_volume_integral_per_zone")
    }

    pub fn compartments_volumes(&self) -> Vec<f64> {
        todo!()
    }

    pub fn get_real_volume(&self) -> &[f64] {
        todo!()
    }
}
