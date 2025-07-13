use crate::{coordinates::CartesianCoordinates, ensight_gold::{
    types::{ElementsType, VolumeElementTypes},
    Part,
}};
const C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM: usize = 20;

#[derive(Default, Debug)]
pub struct VolumeElementData {
    global_id: Vec<Vec<usize>>,
    // part_global_id: Vec<usize>,         // Part GID accessed via voGID
    pub vtype: Vec<VolumeElementTypes>, // Volume element type accessed via voGID
    ids: Vec<usize>,                    // Volume element ID accessed via voGID
    vertices: Vec<usize>,               // List of vertices attached to volume element
    pub xyz: Vec<CartesianCoordinates>,                // Coordinates of center of volume element
    // raz: Vec<f64>,                // Additional coordinates or metadata

    // cell_id: Vec<usize>,          // cID accessed via voGID
    vertices_cell_id: Vec<usize>, // cID associated with each vertex of volume element
    nc_id: Vec<usize>,            // Number of cID per volume element
    compartment_ids: Vec<usize>,    // List of cID in which vertices are
}

impl VolumeElementData {
    pub fn get_vertex_per_element(&self, global_id: usize) -> usize {
        // if self.vtype.len() <= global_id {
        //     return 0;
        // } else {
        //     self.vtype[global_id].to_index()
        // }
        ElementsType::VolumeElementType(self.vtype[global_id])
            .node_count()
            .try_into()
            .unwrap()
    }

    pub fn get_element_and_nvertex(&self, global_id: usize) -> (VolumeElementTypes,usize) {
        // if self.vtype.len() <= global_id {
        //     return 0;
        // } else {
        //     self.vtype[global_id].to_index()
        // }
        let element = self.vtype[global_id];
        (element,ElementsType::VolumeElementType(element)
            .node_count()
            .try_into()
            .unwrap())
    }

    pub fn set_number_cid(&mut self, global_id: usize, n_cid: usize) {
        self.nc_id[global_id] = n_cid;
    }
    pub fn get_number_cid(&self, global_id: usize) -> usize {
        self.nc_id[global_id]
    }

    pub fn resize(&mut self, n_part: usize, n_velement: usize, velement_detail: &[usize]) {
        self.global_id
            .resize(n_part * VolumeElementTypes::NUMBER_OF_TYPES, Vec::new());

        for (vec, &new_len) in self.global_id.iter_mut().zip(velement_detail.iter()) {
            vec.resize(new_len, 0);
        }
        // for i in 0..self.global_id.len() {
        //     self.global_id[i].resize(velement_detail[i], 0);
        // }

        // self.part_global_id.resize(n_velement, 0);
        // self.vtype.resize(n_velement,VolumeElementTypes::Hexa8);
        self.vtype = Vec::with_capacity(n_velement);
        self.ids.resize(n_velement, 0);
        self.nc_id.resize(n_velement, 0);
        // self.cell_id.resize(n_velement, 0);

        self.vertices
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
        self.compartment_ids
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
        self.xyz.resize(n_velement , Default::default());
        // self.raz.resize(n_velement * 3, 0.);

        self.vertices_cell_id
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
    }

    pub fn n_element(&self) -> usize {
        self.ids.len()
    }

    pub fn set_list_compartment_id(&mut self, global_id: usize, k_vertex: usize, val: usize) {
        self.compartment_ids[global_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex] = val;
    }

    pub fn get_list_compartment_id(&self, global_id: usize, k_vertex: usize) -> usize {
        self.compartment_ids[global_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex]
    }

    pub fn set_global_id(
        &mut self,
        i_part: usize,
        element_index: usize,
        volume_element_id: usize,
        val: usize,
    ) {
        self.global_id[VolumeElementTypes::NUMBER_OF_TYPES * i_part + element_index]
            [volume_element_id] = val;
    }

    pub fn get_global_id(
        &self,
        i_part: usize,
        element_index: usize,
        volume_element_id: usize,
    ) -> usize {
        self.global_id[VolumeElementTypes::NUMBER_OF_TYPES * i_part + element_index]
            [volume_element_id]
    }

    pub fn get_vertex_from_vol_global_id(
        &self,
        vol_element_global_id: usize,
        k_vertex: usize,
    ) -> usize {
        self.vertices[vol_element_global_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex]
    }

    fn set_vertex_from_vol_global_id(
        &mut self,
        vol_element_global_id: usize,
        k_vertex: usize,val:usize
    )  {
        self.vertices[vol_element_global_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex]=val;
    }

    pub fn fill_from_part(
        &mut self,
        part_it: (usize, &Part),
        velem_detail: &[usize],
        vertices: &VerticesData,
        ve_counter: &mut usize,
    ) {
        const N_NUMBER_TYPE: usize = VolumeElementTypes::NUMBER_OF_TYPES;
        let (i_part, part) = part_it;
        for element in part.elements.iter() {
            match element.etype {
                ElementsType::VolumeElementType(var) => {
                    let n_vertex = element.etype.node_count() as usize;
                    let n_volume_element = velem_detail[(i_part * N_NUMBER_TYPE) + var.to_index()];

                    let current_vertex_vegid = &vertices.ve_gid[i_part];

                    for ve_id in 0..n_volume_element {
                        let ve_global_id = *ve_counter;
                        *ve_counter += 1;

                        self.set_global_id(i_part, var.to_index(), ve_id, ve_global_id);

                        // self.part_global_id[ve_global_id] = i_part;
                        // self.vtype[ve_global_id] = var;
                        self.vtype.push(var);
                        self.ids[ve_global_id] = ve_id;

                        for k_vertex in 0..n_vertex {
                            // let vtx = element.vertices[ve_id * n_vertex + k_vertex];
                            let vtx = element.get_vertex(ve_id, k_vertex);
                            self.set_vertex_from_vol_global_id(ve_global_id,k_vertex,current_vertex_vegid[vtx-1]);
                            // self.vertices[ve_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex] =
                            //     current_vertex_vegid[vtx - 1];
                        }
                    }
                }
                _ => {
                    // panic!("TODO Not a volume element {:?}",e);
                    continue;
                }
            }
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
            self.xyz[offset..offset + 3].copy_from_slice(part.get_vertex_coordinates_slice(ve_id));
            *vertex_counter += 1;
        }
    }
    pub fn resize(&mut self, n_part: usize, n_vertices: usize, vertex_detail: &[usize]) {
        self.ve_gid.resize(n_part, Vec::new());
        // for i in 0..n_part {
        //     self.ve_gid[i].resize(vertex_detail[i], 0);
        // }

        self.ve_gid.iter_mut().zip(vertex_detail).for_each(
            |(ve,size)|
            {
                ve.resize(*size, 0);
            }
        );

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
