// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    coordinates::CartesianCoordinates,
    ensight_gold::{
        Part,
        types::{ElementsType, VolumeElementTypes},
    },
};
const C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM: usize = 20;

#[derive(Default, Debug)]
pub struct VolumeElementData {
    global_id: Vec<Vec<usize>>,
    // part_global_id: Vec<usize>,         // Part GID accessed via voGID
    vtype: Vec<VolumeElementTypes>, // Volume element type accessed via voGID
    ids: Vec<usize>,                // Volume element ID accessed via voGID
    vertices: Vec<usize>,           // List of vertices attached to volume element
    pub xyz: Vec<CartesianCoordinates>, // Coordinates of center of volume element
    // raz: Vec<f64>,                // Additional coordinates or metadata

    // cell_id: Vec<usize>,          // cID accessed via voGID
    vertices_cell_id: Vec<usize>, // cID associated with each vertex of volume element
    nc_id: Vec<usize>,            // Number of cID per volume element
    compartment_ids: Vec<usize>,  // List of cID in which vertices are
}

impl VolumeElementData {
    pub fn get_vertex_per_element(&self, global_id: usize) -> usize {
        ElementsType::VolumeElementType(self.vtype[global_id])
            .node_count()
            .try_into()
            .unwrap()
    }

    pub fn get_element_and_nvertex(&self, global_id: usize) -> (VolumeElementTypes, usize) {
        let element = self.vtype[global_id];
        (
            element,
            ElementsType::VolumeElementType(element)
                .node_count()
                .try_into()
                .unwrap(),
        )
    }

    pub fn set_number_cid(&mut self, global_id: usize, n_cid: usize) {
        self.nc_id[global_id] = n_cid;
    }
    pub fn get_number_cid(&self, global_id: usize) -> usize {
        self.nc_id[global_id]
    }

    pub fn enumerate_number_id(&self) -> std::iter::Enumerate<std::slice::Iter<'_, usize>> {
        self.nc_id.iter().enumerate()
    }

    pub fn resize(&mut self, n_part: usize, n_velement: usize, velement_detail: &[usize]) {
        self.global_id
            .resize(n_part * VolumeElementTypes::NUMBER_OF_TYPES, Vec::new());

        for (vec, &new_len) in self.global_id.iter_mut().zip(velement_detail.iter()) {
            vec.resize(new_len, 0);
        }
        //Use with_capacity because volument_element
        //doesn't provide default value.
        self.vtype = Vec::with_capacity(n_velement);
        self.ids.resize(n_velement, 0);
        self.nc_id.resize(n_velement, 0);
        self.vertices
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
        self.compartment_ids
            .resize(n_velement * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM, 0);
        self.xyz.resize(n_velement, Default::default());
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
        k_vertex: usize,
        val: usize,
    ) {
        self.vertices[vol_element_global_id * C_MAX_NUMBER_VERTEX_PER_VOLUME_ELEM + k_vertex] = val;
    }

    pub fn fill_from_part(
        &mut self,
        global_volume_element_counter: usize,
        part_it: (usize, &Part),
        velem_detail: &[usize],
        vertices: &VerticesData,
    ) -> usize {
        const N_NUMBER_TYPE: usize = VolumeElementTypes::NUMBER_OF_TYPES;
        let (i_part, part) = part_it;

        //local_vec_counter is the sum of n_element for eaach element in part.
        //Actually can be computed with the iter below
        // let result :usize= part.elements.iter()
        // .filter(|e| matches!(e.etype, ElementsType::VolumeElementType(_)))
        // .map(|vol_element| vol_element.n_elements).sum();

        //But as we perform different operation during the loop it's better to just iterate once.

        //Vec counter logic is not the same as vertice becuase we can't predict the number of itertion
        // inner loop bound si not trividl
        //To return number of volument_element
        //Start from count=n, during loop , count +=m, so return count-n
        let mut local_vec_counter = global_volume_element_counter;

        //mutable state, loop counters, nested loops, and side-effecting methods (self.set_*)
        //the for loop + if let version is better than filter().for_each

        let i_part_base_index = i_part * N_NUMBER_TYPE;

        for element in part.elements.iter() {
            if let ElementsType::VolumeElementType(vol_elem) = element.etype {
                let n_vertex = element.etype.node_count() as usize;
                let element_index = vol_elem.to_index();
                let n_volume_element = velem_detail[i_part_base_index + element_index];

                let current_vertex_in_part = vertices.get_current_vertex_from_part(i_part);

                for ve_id in 0..n_volume_element {
                    let ve_global_id = local_vec_counter;

                    local_vec_counter += 1;

                    self.set_global_id(i_part, element_index, ve_id, ve_global_id);

                    self.vtype.push(vol_elem);
                    self.ids[ve_global_id] = ve_id;

                    for k_vertex in 0..n_vertex {
                        let vtx = element.get_vertex(ve_id, k_vertex);
                        self.set_vertex_from_vol_global_id(
                            ve_global_id,
                            k_vertex,
                            current_vertex_in_part[vtx - 1],
                        );
                    }
                }
            }
        }

        local_vec_counter - global_volume_element_counter
    }
}

//TODO: Remove this and merge with volume ?
#[derive(Default)]
pub struct VerticesData {
    //Actually only used to construct volume data
    //TODO: Make this temp varialbe ?
    ve_gid: Vec<Vec<usize>>, // Access to veGID by part and vertex

    // part_id: Vec<usize>,     // Access to vertex partID from veGID
    // ve_id: Vec<usize>,   // Access to vertex veID from veGID
    xyz: Vec<f64>, // Vertices coordinates
                   // vertex_c_id: Vec<usize>, // Access to vertex cID from veGID
}

impl VerticesData {
    pub fn fill_from_part(
        &mut self,
        vertex_counter: usize,
        vertex_detail: &[usize],
        part_it: (usize, &Part),
    ) -> usize {
        let (i_part, part) = part_it;

        let mut vertex_global_identifier = vertex_counter;
        #[allow(clippy::explicit_counter_loop)]
        for ve_id in 0..vertex_detail[i_part] {
            self.ve_gid[i_part][ve_id] = vertex_global_identifier;
            let offset = vertex_global_identifier * 3;
            self.xyz[offset..offset + 3].copy_from_slice(part.get_vertex_coordinates_slice(ve_id));
            vertex_global_identifier += 1;
        }
        vertex_detail[i_part]
    }
    pub fn resize(&mut self, n_vertices: usize, vertex_detail: &[usize]) {
        self.ve_gid = vertex_detail
            .iter()
            .map(|n_vertex_p_part| vec![0; *n_vertex_p_part])
            .collect();

        // self.part_id.resize(n_vertices, 0);
        // self.ve_id.resize(n_vertices, 0);

        self.xyz.resize(n_vertices * 3, 0.);
        // self.vertex_c_id.resize(n_vertices, 0);
    }

    pub fn n_vertex(&self) -> usize {
        self.xyz.len() / 3
    }

    fn get_current_vertex_from_part(&self, part_id: usize) -> &[usize] {
        &self.ve_gid[part_id]
    }

    pub fn get_slice_xyz(&self, global_id: usize) -> &[f64; 3] {
        let offset = global_id * 3;
        self.xyz[offset..offset + 3]
            .try_into()
            .expect("Slice with exactly 3 elements")
    }
    pub fn get_radius_from_global_id(&self, vertex_global_id: usize) -> f64 {
        let offset = vertex_global_id * 3;
        self.xyz[offset].hypot(self.xyz[offset + 1])
    }
}
