use std::{collections::BTreeSet, sync::Arc};

use crate::{ensight_gold::{self, types::ElementsType}, grid::{cylindrical_index, get_mesh, CompartmentMesh, Coords3, CylindricalAxis, MeshType}, model::data::{VerticesData, VolumeElementData}};


pub struct CMGeometry {
    n_zones: usize,
    pub vertices: VerticesData,
    pub volume_elements: VolumeElementData,
    grid:Option<Box<dyn CompartmentMesh>>,
  
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

        axis.iter_mut().for_each(|ax| ax.step = (ax.max_range-ax.min_range)/(ax.n_range as f64));

        self.grid = Some(get_mesh(mesh_type, axis));
    }

    fn compute_centroid(&self, volume_element_global_id: usize, n_vertex: usize) -> Coords3 {
        todo!("centroid")
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
                    let vertex_global_id = self.volume_elements.get_vertex_from_vol_global_id(vol_element_global_id,k_vertex);
                    // self.vertices.ve_id[vertex_global_id]
                    vertices_id[vertex_global_id]
                })
                .collect();
            self.volume_elements.set_number_cid(vol_element_global_id,unique_cids.len());
        }
    }

    pub fn init(
        n_div: [usize; 3],
        geometry: Arc<ensight_gold::Geometry>,
        mesh_type: crate::grid::MeshType,
    ) -> Self {
        let mut cm_geometry = Self {
            n_zones: n_div.iter().product::<usize>(),
            vertices: Default::default(),
            volume_elements: Default::default(),
            grid:None,
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

    fn get_grid(&self)->Option<&dyn CompartmentMesh>{
        self.grid.as_deref()
    }
}
