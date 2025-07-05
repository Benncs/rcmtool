use std::{collections::BTreeSet, sync::Arc};

use crate::{
    ensight_gold::{
        self,
        types::{ElementsType, VolumeElementTypes},
        Part,
    },
    grid::{
        cylindrical_index, get_mesh, AxisDescriptor, CompartmentMesh, Coords3, CylindricalAxis,
        MeshType,
    },
    model::scalar::Scalar,
    CoreError,
};
mod data;
pub mod scalar;
use data::*;
const C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM: usize = 20;

pub struct CMGeometry {
    n_zones: usize,
    vertices: VerticesData,
    volume_elements: VolumeElementData,
}

impl CMGeometry {
    fn fill_detail(&mut self, geometry: &Arc<ensight_gold::Geometry>) -> (Vec<usize>, Vec<usize>) {
        let n_number_type = ensight_gold::types::VolumeElementTypes::number_of_types();
        let n_part = geometry.number_of_part();
        let mut vertex_detail = Vec::<usize>::with_capacity(n_part);
        let mut velem_detail = vec![0; n_part * n_number_type];
        let mut n_vertex_total = 0;
        let mut n_volume_elements_total = 0;

        for (i, part) in geometry.parts.iter().enumerate() {
            n_vertex_total += part.n_vertex;
            vertex_detail.push(part.n_vertex);

            for element in &part.elements {
                match element.etype {
                    ElementsType::VolumeElementType(_) => {
                        let n_nodes = element.etype.node_count() as usize;
                        n_volume_elements_total += element.n_elements;
                        velem_detail[(i * n_number_type) + n_nodes] += element.n_elements;
                    }
                    e => {
                        // panic!("TODO Not a volume element {:?}",e);
                        continue;
                    }
                }
            }
        }

        self.vertices.resize(n_part, n_vertex_total, &vertex_detail);

        self.volume_elements
            .resize(n_part, n_volume_elements_total, &velem_detail);

        (vertex_detail, velem_detail)
    }

    fn init_cm_grid(&self, n_div: [usize; 3], mesh_type: MeshType) -> Box<dyn CompartmentMesh> {
        let mut axis: [super::grid::AxisDescriptor; 3] = Default::default();

        axis.iter_mut().zip(n_div).for_each(|(ax, div)| {
            ax.n_range = div;
        });

        for vertex_global_id in 0..self.vertices.n_vertex() {
            axis.iter_mut()
                .zip(self.vertices.get_slice_xyz(vertex_global_id))
                .for_each(|(axe, &vertex)| {
                    axe.min_range = axe.min_range.min(vertex);
                    axe.max_range = axe.max_range.max(vertex);
                });
            if mesh_type == MeshType::Cylindrical {
                let offset = vertex_global_id * 3;
                let radius = self.vertices.xyz[offset].hypot(self.vertices.xyz[offset + 1]);
                axis[cylindrical_index(CylindricalAxis::R)].max_range =
                    axis[0].max_range.max(radius);
            }
        }
        if mesh_type == MeshType::Cylindrical {
            axis[cylindrical_index(CylindricalAxis::R)].min_range = 0.;
        }

        get_mesh(mesh_type, axis)
    }

    fn compute_centroid(&self, volume_element_global_id: usize, n_vertex: usize) -> Coords3 {
        todo!("centroid")
    }

    fn detect_compartment(&mut self, n_div: [usize; 3], mesh_type: MeshType) {
        let grid = self.init_cm_grid(n_div, mesh_type);

        // let vertices_id: Vec<_> = (0..self.vertices.n_vertex())
        //     .map(|global_id| {
        //         grid.cell_from_coordinates(self.vertices.get_slice_xyz(global_id))
        //             .unwrap()
        //     })
        //     .collect();

        for vol_element_global_id in 0..self.volume_elements.n_element() {
            let n_vertex = self
                .volume_elements
                .get_vertex_per_element(vol_element_global_id);

            let unique_cids: BTreeSet<_> = (0..n_vertex)
                .map(|k_vertex| {
                    let vertex_global_id = self.volume_elements.get_vertex_from_vol_global_id(vol_element_global_id,k_vertex);
                    self.vertices.ve_id[vertex_global_id]
                })
                .collect();
            self.volume_elements.set_number_cid(vol_element_global_id,unique_cids.len());
        }
    }

    pub fn init(
        n_div: [usize; 3],
        geometry: Arc<ensight_gold::Geometry>,
        mesh_type: super::grid::MeshType,
    ) -> Self {
        let mut cm_geometry = Self {
            n_zones: n_div.iter().product::<usize>(),
            vertices: Default::default(),
            volume_elements: Default::default(),
        };

        let (vertex_detail, velem_detail) = cm_geometry.fill_detail(&geometry);

        let mut vertex_counter = 0;
        let mut ve_counter = 0;

        for part_it in geometry.parts.iter().enumerate() {
            cm_geometry
                .vertices
                .fill_from_part(&mut vertex_counter, &vertex_detail, part_it);

            cm_geometry.volume_elements.fill_from_part(
                part_it,
                &velem_detail,
                &cm_geometry.vertices,
                &mut ve_counter,
            )
        }

        cm_geometry.detect_compartment(n_div, mesh_type);

        cm_geometry
    }
}

pub struct CMModel {
    geometry: CMGeometry,
}

impl CMModel {}

impl CMModel {
    pub fn init(geometry: CMGeometry) -> Self {
        Self { geometry }
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
        todo!()
    }

    pub fn compartments_volumes(&self) -> Vec<f64> {
        todo!()
    }

    pub fn get_real_volume(&self) -> &[f64] {
        todo!()
    }
}
