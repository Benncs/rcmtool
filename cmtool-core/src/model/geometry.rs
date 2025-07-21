use std::{collections::BTreeSet, default, sync::Arc};

use crate::{
    coordinates::{CartesianCoordinates, Coords3},
    ensight_gold::{self, types::ElementsType},
    grid::{
        CompartmentMesh, CylindricalAxis, MeshType, NeighborDirection, cylindrical_index, get_mesh,
    },
    model::{
        CountVolumeElement,
        data::{VerticesData, VolumeElementData},
        interfaces::AInterfacesInfo,
    },
    utils::compute_centroid,
};

pub struct CMGeometry {
    pub vertices: VerticesData,
    pub volume_elements: VolumeElementData,
    grid: Option<Box<dyn CompartmentMesh>>,
    pub mesh_type: crate::grid::MeshType,
}

//Mutable
impl CMGeometry {
    fn fill_detail(&mut self, geometry: &Arc<ensight_gold::Geometry>) -> (Vec<usize>, Vec<usize>) {
        let n_number_type = ensight_gold::types::VolumeElementTypes::NUMBER_OF_TYPES;
        let n_part = geometry.number_of_part();
        let mut vertex_detail = Vec::<usize>::with_capacity(n_part);
        let mut velem_detail = vec![0; n_part * n_number_type];
        let mut n_vertex_total = 0;
        let mut n_volume_elements_total = 0;

        for (i_part, part) in geometry.parts.iter().enumerate() {
            n_vertex_total += part.n_vertex;
            vertex_detail.push(part.n_vertex);
            let base_index = i_part * n_number_type;
            for element in &part.elements {
                if let ElementsType::VolumeElementType(vol_element) = element.etype {
                    let index_element = vol_element.to_index();
                    n_volume_elements_total += element.n_elements;
                    velem_detail[base_index + index_element] += element.n_elements;
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
                let radius = self.vertices.get_radius_from_global_id(vertex_global_id);
                axis[cylindrical_index(CylindricalAxis::R)].max_range =
                    axis[0].max_range.max(radius);
            }
        }
        if mesh_type == MeshType::Cylindrical {
            axis[cylindrical_index(CylindricalAxis::R)].min_range = 0.;
        }

        // axis.iter_mut()
        //     .for_each(|ax| ax.step = (ax.max_range - ax.min_range) / (ax.n_range as f64));

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

            self.volume_elements.xyz[vol_element_global_id] =
                self.get_element_centroid(vol_element_global_id, n_vertex);

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
                    .set_list_compartment_id(vol_element_global_id, index, c_id);
            }
        }
    }

    pub fn get_element_centroid(
        &self,
        vol_element_global_id: usize,
        n_vertex: usize,
    ) -> CartesianCoordinates {
        //This iter is lazy as we map without operation
        let iter = (0..n_vertex).map(|k| {
            let vertex_id = self
                .volume_elements
                .get_vertex_from_vol_global_id(vol_element_global_id, k);
            self.vertices.get_slice_xyz(vertex_id)
        });

        compute_centroid(iter)
    }
}

impl CMGeometry {
    pub fn n_zone(&self) -> usize {
        self.grid.as_ref().unwrap().number_cell()
    }

    pub(super) fn interface_iterator(&self, mut f: impl FnMut(usize, usize, usize, usize)) {
        for vol_element_global_id in 0..self.volume_elements.n_element() {
            let interface_cid_0 = self
                .volume_elements
                .get_list_compartment_id(vol_element_global_id, 0);

            let n_cid = self.volume_elements.get_number_cid(vol_element_global_id);

            for k_vertex in 0..n_cid {
                let interface_cid_k = self
                    .volume_elements
                    .get_list_compartment_id(vol_element_global_id, k_vertex);
                f(
                    vol_element_global_id,
                    interface_cid_0,
                    interface_cid_k,
                    k_vertex,
                );
            }
        }
    }

    pub fn get_count_volume_element_first_pass(&self) -> CountVolumeElement {
        let mut count = CountVolumeElement::new(self.n_zone());

        let functor = |_vol_element_global_id: usize,
                       interface_cid_0: usize,
                       interface_cid_k: usize,
                       k_vertex: usize| {
            count.incr_compartment(interface_cid_k);

            if k_vertex >= 1 {
                match self
                    .grid
                    .as_ref()
                    .unwrap()
                    .are_cell_neighbor(interface_cid_0, interface_cid_k)
                {
                    NeighborDirection::NotNeighbors => {
                        //NOP
                    }
                    neighbors => {
                        let (id1, id2) = neighbors.ordered_pair(interface_cid_0, interface_cid_k);

                        count.incr_interface(id1, id2);
                    }
                }
            }
        };

        self.interface_iterator(functor);

        count
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
            mesh_type,
        };

        let (vertex_detail, velem_detail) = cm_geometry.fill_detail(&geometry);

        let mut global_vertex_counter = 0;
        let mut global_volume_element_counter = 0;

        for part_it in geometry.parts.iter().enumerate() {
            global_vertex_counter +=
                cm_geometry
                    .vertices
                    .fill_from_part(global_vertex_counter, &vertex_detail, part_it);

            cm_geometry.volume_elements.fill_from_part(
                part_it,
                &velem_detail,
                &cm_geometry.vertices,
                &mut global_volume_element_counter,
            )
        }

        cm_geometry.detect_compartment(n_div, mesh_type);

        cm_geometry
    }

    pub fn get_grid(&self) -> Option<&dyn CompartmentMesh> {
        self.grid.as_deref()
    }
}
