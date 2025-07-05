use crate::ensight_gold::{
    types::{ElementsType, VolumeElementTypes},
    Part,
};
const C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM: usize = 20;

#[derive(Default, Debug)]
pub struct VolumeElementData {
    global_id: Vec<Vec<usize>>,
    part_global_id: Vec<usize>,     // Part GID accessed via voGID
    vtype: Vec<VolumeElementTypes>, // Volume element type accessed via voGID
    ids: Vec<usize>,                // Volume element ID accessed via voGID
    vertices: Vec<usize>,           // List of vertices attached to volume element
    // xyz: Vec<f64>,                // Coordinates of center of volume element
    // raz: Vec<f64>,                // Additional coordinates or metadata

    // cell_id: Vec<usize>,          // cID accessed via voGID
    vertices_cell_id: Vec<usize>, // cID associated with each vertex of volume element
    nc_id: Vec<usize>,            // Number of cID per volume element
    limit_cell_id: Vec<usize>,    // List of cID in which vertices are
}

impl VolumeElementData {
    pub fn get_vertex_per_element(&self, global_id: usize) -> usize {
        if self.vtype.len() <= global_id {
            return 0;
        } else {
            self.vtype[global_id].to_index()
        }
    }

    pub fn set_number_cid(&mut self, global_id: usize, n_cid: usize) {
        self.nc_id[global_id] = n_cid;
    }

    pub fn resize(&mut self, n_part: usize, n_velement: usize, velement_detail: &[usize]) {
        self.global_id
            .resize(n_part * VolumeElementTypes::number_of_types(), Vec::new());
        for i in 0..self.global_id.len() {
            self.global_id[i].resize(velement_detail[i], 0);
        }

        self.part_global_id.resize(n_velement, 0);
        // self.vtype.resize(n_velement,VolumeElementTypes::Hexa8);
        self.vtype = Vec::with_capacity(n_velement);
        self.ids.resize(n_velement, 0);
        self.nc_id.resize(n_velement, 0);
        // self.cell_id.resize(n_velement, 0);

        self.vertices
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
        self.limit_cell_id
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
        // self.xyz.resize(n_velement * 3, 0.);
        // self.raz.resize(n_velement * 3, 0.);

        self.vertices_cell_id
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
    }

    pub fn n_element(&self) -> usize {
        self.ids.len()
    }

    pub fn set_global_id(&mut self, i_part: usize, element_index: usize, ve_id: usize, val: usize) {
        self.global_id[VolumeElementTypes::number_of_types() * i_part + element_index][ve_id] = val;
    }

    pub fn get_vertex_from_vol_global_id(
        &self,
        vol_element_global_id: usize,
        k_vertex: usize,
    ) -> usize {
        vol_element_global_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex
    }

    pub fn fill_from_part(
        &mut self,
        part_it: (usize, &Part),
        velem_detail: &[usize],
        vertices: &VerticesData,
        ve_counter: &mut usize,
    ) {
        let n_number_type = VolumeElementTypes::number_of_types();
        let (i_part, part) = part_it;
        for (i, element) in part.elements.iter().enumerate() {
            match element.etype {
                ElementsType::VolumeElementType(var) => {
                    let n_vertex = element.etype.node_count() as usize;
                    let n_volume_element = velem_detail[(i * n_number_type) + var.to_index()];

                    let current_vertex_vegid = &vertices.ve_gid[i_part];

                    for ve_id in 0..n_volume_element {
                        let ve_global_id = *ve_counter;

                        self.set_global_id(i_part, n_vertex, ve_id, ve_global_id);

                        self.part_global_id[ve_global_id] = i_part;
                        // self.vtype[ve_global_id] = var;
                        self.vtype.push(var);
                        self.ids[ve_global_id] = ve_id;

                        for k_vertex in 0..n_vertex {
                            let vtx = element.vertices[ve_id * n_vertex + k_vertex];
                            self.vertices[ve_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex] =
                                current_vertex_vegid[vtx - 1];
                        }
                    }
                }
                _ => {
                    // panic!("TODO Not a volume element {:?}",e);
                    continue;
                }
            }
            *ve_counter += 1;
        }
    }
}

#[derive(Default, Debug)]
pub struct VerticesData {
    ve_gid: Vec<Vec<usize>>, // Access to veGID by part and vertex
    part_id: Vec<usize>,     // Access to vertex partID from veGID
    pub ve_id: Vec<usize>,   // Access to vertex veID from veGID
    pub xyz: Vec<f64>,       // Vertices coordinates
    vertex_c_id: Vec<usize>, // Access to vertex cID from veGID
}

impl VerticesData {
    pub fn fill_from_part(
        &mut self,
        vertex_counter: &mut usize,
        vertex_detail: &[usize],
        part_it: (usize, &Part),
    ) {
        let (i_part, part) = part_it;
        for ve_id in 0..vertex_detail[i_part] {
            let vertex_global_identifier = *vertex_counter;

            self.ve_gid[i_part][ve_id] = vertex_global_identifier;
            self.part_id[*vertex_counter] = i_part;
            self.ve_id[vertex_global_identifier] = ve_id;

            let offset = (*vertex_counter) * 3;
            let coords = part.get_vertex_coordinates_vec(ve_id);
            self.xyz[offset] = coords[0];
            self.xyz[offset + 1] = coords[1];
            self.xyz[offset + 2] = coords[2];
            *vertex_counter += 1;
        }
    }
    pub fn resize(&mut self, n_part: usize, n_vertices: usize, vertex_detail: &[usize]) {
        self.ve_gid.resize(n_part, Vec::new());
        for i in 0..n_part {
            self.ve_gid[i].resize(vertex_detail[i], 0);
        }

        self.part_id.resize(n_vertices, 0);
        self.ve_id.resize(n_vertices, 0);
        self.xyz.resize(n_vertices * 3, 0.);
        self.vertex_c_id.resize(n_vertices, 0);
    }

    pub fn n_vertex(&self) -> usize {
        self.ve_id.len()
    }
    pub fn get_slice_xyz(&self, global_id: usize) -> &[f64; 3] {
        let offset = global_id * 3;
        self.xyz[offset..offset + 3]
            .try_into()
            .expect("Slice with exactly 3 elements")
    }
}
