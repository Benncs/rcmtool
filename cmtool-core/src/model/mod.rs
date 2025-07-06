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

        let geometry = self.geometry.as_ref();

        let mut tmp_count_k_element: Vec<usize> = vec![0; self.geometry.n_zone()];

        for volume_element_global_id in 0..self.geometry.volume_elements.n_element() {
            let n_compartment_in_velem = geometry
                .volume_elements
                .get_number_cid(volume_element_global_id);

            let n_vertex = geometry
                .volume_elements
                .get_vertex_per_element(volume_element_global_id);

            let vtype: VolumeElementTypes =
                geometry.volume_elements.vtype[volume_element_global_id];

            local_vertices.clear();
            local_vertices.resize(n_vertex, Default::default());
            for i_compartment_in_velem in 0..n_compartment_in_velem {
                let compartment_id = geometry
                    .volume_elements
                    .get_list_compartment_id(volume_element_global_id, i_compartment_in_velem);

                let k_element = tmp_count_k_element[compartment_id];
                tmp_count_k_element[compartment_id] += 1;

                for (k_vertex, local_vertex) in local_vertices.iter_mut().enumerate() {
                    let vertex_global_id = geometry
                        .volume_elements
                        .get_vertex_from_vol_global_id(volume_element_global_id, k_vertex);
                    *local_vertex = geometry.vertices.get_slice_xyz(vertex_global_id).to_owned();
                }

                let volume = compute_volume(&local_vertices, vtype).unwrap()
                    / (n_compartment_in_velem as f64);
                self.c_info.volumes[compartment_id][k_element] = ElementVolumeInfo {
                    global_id: volume_element_global_id,
                    volume,
                };
                assert!(volume > 0.);
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

        scalar_field.values = vec![(0.).into(); self.geometry.n_zone()];

        for (volumes_i, field) in self.c_info.volumes.iter().zip(&mut scalar_field.values) {
            field.value = volumes_i
                .iter()
                .fold(0.0, |acc, ElementVolumeInfo { global_id, volume }| {
                    acc + scalar[*global_id] * volume
                });
        }

        Ok(scalar_field)
    }

    pub fn compartments_volumes(&self) -> Vec<f64> {
        todo!()
    }

    pub fn get_real_volume(&self) -> &[f64] {
        todo!()
    }
}
