use std::sync::Arc;

use crate::{
    ensight_gold::{
        self,
        types::{ElementsType, VolumeElementTypes},
    },
    model::scalar::Scalar,
};
pub mod scalar;
const c_max_number_vertex_per_volume_elem: usize = 20;
#[derive(Default, Debug)]
struct VolumeElementData {
    global_id: Vec<Vec<usize>>,
    part_global_id: Vec<usize>,   // Part GID accessed via voGID
    vtype: Vec<usize>,            // Volume element type accessed via voGID
    ids: Vec<usize>,              // Volume element ID accessed via voGID
    vertices: Vec<usize>,         // List of vertices attached to volume element
    xyz: Vec<f64>,                // Coordinates of center of volume element
    raz: Vec<f64>,                // Additional coordinates or metadata
    cell_id: Vec<usize>,          // cID accessed via voGID
    vertices_cell_id: Vec<usize>, // cID associated with each vertex of volume element
    nc_id: Vec<usize>,            // Number of cID per volume element
    limit_cell_id: Vec<usize>,    // List of cID in which vertices are
}

#[derive(Default, Debug)]
struct VerticesData {
    ve_gid: Vec<Vec<usize>>, // Access to veGID by part and vertex
    part_id: Vec<usize>,     // Access to vertex partID from veGID
    ve_id: Vec<usize>,       // Access to vertex veID from veGID
    xyz: Vec<f64>,           // Vertices coordinates
    vertex_c_id: Vec<usize>, // Access to vertex cID from veGID
}

pub struct CMGeometry {
    n_zones: usize,
    vertices: VerticesData,
    volume_elements: VolumeElementData,
}

impl CMGeometry {
    pub fn init(n_div: [usize; 3], geometry: Arc<ensight_gold::Geometry>) -> Self {
        let mut cm_geometry = Self {
            n_zones: n_div.iter().product::<usize>(),
            vertices: Default::default(),
            volume_elements: Default::default(),
        };

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
                        n_volume_elements_total += element.n_elements as usize;
                        velem_detail[(i * n_number_type) + n_nodes] += element.n_elements;
                    }
                    e => {
                        // panic!("TODO Not a volume element {:?}",e);
                        continue;
                    }
                }
            }
        }

        cm_geometry
            .vertices
            .resize(n_part, n_vertex_total, &vertex_detail);

        cm_geometry
            .volume_elements
            .resize(n_part, n_volume_elements_total, &velem_detail);

        let mut vertex_counter = 0;
        let mut ve_counter = 0;

        for (i_part, part) in geometry.parts.iter().enumerate() {
            for ve_id in 0..vertex_detail[i_part] {
                let vertex_global_identifier = vertex_counter;

                cm_geometry.vertices.ve_gid[i_part][ve_id] = vertex_global_identifier;
                cm_geometry.vertices.part_id[vertex_counter] = i_part;
                cm_geometry.vertices.ve_id[vertex_global_identifier] = ve_id;

                let offset = vertex_counter * 3;
                let coords = part.get_vertex_coordinates_vec(ve_id);
                cm_geometry.vertices.xyz[offset] = coords[0];
                cm_geometry.vertices.xyz[offset + 1] = coords[1];
                cm_geometry.vertices.xyz[offset + 2] = coords[2];
                vertex_counter += 1;
            }

            for (i, element) in part.elements.iter().enumerate() {
                match element.etype {
                    ElementsType::VolumeElementType(var) => {
                        let n_vertex = element.etype.node_count() as usize;
                        let n_volume_element = velem_detail[(i * n_number_type) + var.to_index()];

                        let current_vertex_vegid = &cm_geometry.vertices.ve_gid[i_part];

                        for ve_id in 0..n_volume_element {
                            let ve_global_id = ve_counter;

                            cm_geometry.volume_elements.set_global_id(
                                i_part,
                                n_vertex,
                                ve_id,
                                ve_global_id,
                            );

                            cm_geometry.volume_elements.part_global_id[ve_global_id] = i_part;
                            cm_geometry.volume_elements.vtype[ve_global_id] = n_vertex;
                            cm_geometry.volume_elements.ids[ve_global_id] = ve_id;

                            for k_vertex in 0..n_vertex {
                                let vtx = element.vertices[ve_id * n_vertex + k_vertex];
                                cm_geometry.volume_elements.vertices
                                    [ve_id * c_max_number_vertex_per_volume_elem + k_vertex] =
                                    current_vertex_vegid[vtx - 1];
                            }

                            ve_counter += 1;
                        }
                    }
                    _ => {
                        // panic!("TODO Not a volume element {:?}",e);
                        continue;
                    }
                }
            }
        }

        todo!("DetectCompartments");

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
    ) -> Result<cmtool_data::RawDataScalar, ()> {
        todo!()
    }

    pub fn compartments_volumes(&self) -> Vec<f64> {
        todo!()
    }

    pub fn get_real_volume(&self) -> &[f64] {
        todo!()
    }
}

impl VerticesData {
    fn resize(&mut self, n_part: usize, n_vertices: usize, vertex_detail: &[usize]) {
        self.ve_gid.resize(n_part, Vec::new());
        for i in 0..n_part {
            self.ve_gid[i].resize(vertex_detail[i], 0);
        }

        self.part_id.resize(n_vertices, 0);
        self.ve_id.resize(n_vertices, 0);
        self.xyz.resize(n_vertices * 3, 0.);
        self.vertex_c_id.resize(n_vertices, 0);
    }
}

impl VolumeElementData {
    fn resize(&mut self, n_part: usize, n_velement: usize, velement_detail: &[usize]) {
        self.global_id
            .resize(n_part * VolumeElementTypes::number_of_types(), Vec::new());
        for i in 0..self.global_id.len() {
            self.global_id[i].resize(velement_detail[i], 0);
        }

        self.part_global_id.resize(n_velement, 0);
        self.vtype.resize(n_velement, 0);
        self.ids.resize(n_velement, 0);
        self.nc_id.resize(n_velement, 0);
        self.cell_id.resize(n_velement, 0);

        self.vertices
            .resize(n_velement * c_max_number_vertex_per_volume_elem, 0);
        self.limit_cell_id
            .resize(n_velement * c_max_number_vertex_per_volume_elem, 0);
        self.xyz.resize(n_velement * 3, 0.);
        self.raz.resize(n_velement * 3, 0.);

        self.vertices_cell_id
            .resize(n_velement * c_max_number_vertex_per_volume_elem, 0);
    }

    fn set_global_id(&mut self, i_part: usize, element_index: usize, ve_id: usize, val: usize) {
        self.global_id[VolumeElementTypes::number_of_types() * i_part + element_index][ve_id] = val;
    }
}
