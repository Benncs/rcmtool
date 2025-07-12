use crate::{
    ensight_gold::types::VolumeElementTypes,
    grid::MeshType,
    model::compartments::{
        AInterfacesInfo, CompartmentInfo, CountVolumeElement, ElementVolumeInfo, InterfaceArea,
        InterfaceFlow, InterfaceInfo,
    },
    utils::{self, compute_volume},
    CoreError,
};
use std::sync::Arc;
mod data;
use cmtool_data::{RawDataFlux, RawDataScalar};
mod compartments;
mod geometry;
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

        model.fill_c_info();

        model
    }

    fn fill_interfaces(&mut self) {
        let geometry = self.geometry.as_ref();
        let n_zones = geometry.n_zone();
        let mut interface_counter = 0;
        for source_id in 0..n_zones {
            for target_id in 0..n_zones {
                // let index = source_id*n_zones+target_id;
                let interface_id = interface_counter;
                interface_counter += 1;
                self.interfaces.info[interface_id] = InterfaceInfo {
                    source_id,
                    target_id,
                    global_id: interface_id,
                };
            }
        }
    }

    fn fill_c_info(&mut self) {
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
                assert!(volume >= 0.);
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

    pub fn export_flux_through_limits(
        &self,
        vector: Vector,
    ) -> Result<cmtool_data::RawDataFlux, CoreError> {
        let n_fluxes = self.interfaces.n_facet.len();
        let mut flux_field = RawDataFlux::new(self.geometry.n_zone(), n_fluxes);

        let mut flows: Vec<InterfaceFlow> = vec![Default::default(); n_fluxes];

        for (i_interface, flow) in flows.iter_mut().enumerate() {
            let coords = if self.geometry.mesh_type == MeshType::Cylindrical {
                let centroid = [0., 0., 0.];
                utils::vector_cartesian_to_cylindrical(vector.get_slice_xyz(i_interface), centroid)
            } else {
                vector.get_slice_xyz(i_interface).to_owned()
            };

            let InterfaceArea { area, axis } = &self.interfaces.area[i_interface];
            let f = coords[*axis] * area;

            match f > 0. {
                true => flow.source_flow += f,
                false => flow.target_flow += f.abs(),
            }
        }

        for (i_interface, rd) in flux_field.fluxes.iter_mut().enumerate() {
            let InterfaceInfo {
                global_id: _,
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
        println!(
            "Creating scalar with {} compartment",
            self.geometry.n_zone()
        );
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
