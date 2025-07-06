use std::{collections::BTreeSet, sync::Arc};

use crate::{
    ensight_gold::{self, types::ElementsType},
    grid::{
        cylindrical_index, get_mesh, CompartmentMesh, CylindricalAxis, MeshType,
        NeighborDirection,
    },
    model::{
        data::{VerticesData, VolumeElementData},
        CountVolumeElement,
    }, utils::Coords3,
};

pub struct CMGeometry {
    pub vertices: VerticesData,
    pub volume_elements: VolumeElementData,
    grid: Option<Box<dyn CompartmentMesh>>,
}

//Mutable
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
                    ElementsType::VolumeElementType(e) => {
                        // let n_nodes = element.etype.node_count() as usize;

                        let index_element = e.to_index();
                        n_volume_elements_total += element.n_elements;
                        velem_detail[(i * n_number_type) + index_element] += element.n_elements;
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
    fn init_cm_grid(&mut self, n_div: [usize; 3], mesh_type: MeshType) {
        let mut axis: [crate::grid::AxisDescriptor; 3] = Default::default();

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

        axis.iter_mut()
            .for_each(|ax| ax.step = (ax.max_range - ax.min_range) / (ax.n_range as f64));

        self.grid = Some(get_mesh(mesh_type, axis));
    }

    fn detect_compartment(&mut self, n_div: [usize; 3], mesh_type: MeshType) {
        self.init_cm_grid(n_div, mesh_type);

        let grid = self.grid.as_ref().unwrap();

        let vertices_id: Vec<_> = (0..self.vertices.n_vertex())
            .map(|global_id| {
                grid.cell_from_coordinates(self.vertices.get_slice_xyz(global_id))
                    .unwrap()
            })
            .collect();

        for vol_element_global_id in 0..self.volume_elements.n_element() {
            let n_vertex = self
                .volume_elements
                .get_vertex_per_element(vol_element_global_id);

            let unique_cids: BTreeSet<_> = (0..n_vertex)
                .map(|k_vertex| {
                    let vertex_global_id = self
                        .volume_elements
                        .get_vertex_from_vol_global_id(vol_element_global_id, k_vertex);
                    // self.vertices.ve_id[vertex_global_id]
                    vertices_id[vertex_global_id]
                })
                .collect();
            self.volume_elements
                .set_number_cid(vol_element_global_id, unique_cids.len());

            for (index, c_id) in unique_cids.into_iter().enumerate() {
                self.volume_elements
                    .set_limit_cell_id(vol_element_global_id, index, c_id);
            }
        }
    }

}

impl CMGeometry {
    pub fn n_zone(&self) -> usize {
        self.grid.as_ref().unwrap().number_cell()
    }

    pub fn get_count_volume_element(&self) -> CountVolumeElement {
        let mut count = CountVolumeElement::new(self.n_zone());
        const INVALID_CELL_ID: usize = 0;
        for vol_element_global_id in 0..self.volume_elements.n_element() {
            let interface_cid_0 = self
                .volume_elements
                .get_limit_cell_id(vol_element_global_id, 0);

            for k_vertex in 0..self.volume_elements.get_number_cid(vol_element_global_id) {
                let interface_cid_k = self
                    .volume_elements
                    .get_limit_cell_id(vol_element_global_id, k_vertex);

                count.incr_compartment(interface_cid_k);

                if k_vertex >= 1 {
                    match self
                        .grid
                        .as_ref()
                        .unwrap()
                        .are_cell_neighbor(interface_cid_0, interface_cid_k)
                    {
                        NeighborDirection::NotNeighbors => continue,
                        neighbors => {
                            let (id1, id2) =
                                neighbors.ordered_pair(interface_cid_0, interface_cid_k);
                            count.incr_interface(id1, id2);
                        }
                    }
                }
            }
        }

        count
    }

    // pub fn volume_element_per_compartment(&self) -> Vec<usize> {
    //     let mut number_volume_element_per_zone = vec![0; self.n_zones];
    //     for vol_element_global_id in 0..self.volume_elements.n_element() {
    //         for i_cell_id in 0..self.volume_elements.get_number_cid(vol_element_global_id) {
    //             let cell_id = self
    //                 .volume_elements
    //                 .get_limit_cell_id(vol_element_global_id, i_cell_id);
    //             number_volume_element_per_zone[cell_id] += 1;
    //         }
    //     }
    //     number_volume_element_per_zone
    // }

    // pub fn volume_element_at_compartment_interface(&self) {}

    fn compute_centroid(&self, volume_element_global_id: usize, n_vertex: usize) -> Coords3 {
        todo!("centroid")
    }
    pub fn init(
        n_div: [usize; 3],
        geometry: Arc<ensight_gold::Geometry>,
        mesh_type: crate::grid::MeshType,
    ) -> Self {
        let mut cm_geometry = Self {
            vertices: Default::default(),
            volume_elements: Default::default(),
            grid: None,
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

    fn get_grid(&self) -> Option<&dyn CompartmentMesh> {
        self.grid.as_deref()
    }
}
