// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    ensight_gold::{
        reader::EnsightGoldReader,
        variable::{PerElementVariable, VarTypeReader},
    },
    utils,
};

pub(crate) struct VectorReader;

impl VarTypeReader for VectorReader {
    type VarType = Vec<f32>;
    fn read_elements(
        reader: &mut EnsightGoldReader,
        element: &super::geo::MeshElementType,
    ) -> std::io::Result<Self::VarType> {
        let mut flat_data = vec![0.; element.n_elements * 3];

        for i_xyz in 0..3 {
            for i_vertex in 0..element.n_elements {
                flat_data[utils::linear_index_coordinates_matrix(i_vertex, i_xyz)] =
                    reader.read_f32()?;
            }
        }

        Ok(flat_data)
    }
}
pub(crate) type VectorField = PerElementVariable<VectorReader>;

impl VectorField {
    pub fn get_xyz(&self, i_part: usize, i_mesh_element_type: usize, mesh_cell: usize) -> [f64; 3] {
        let flat = &self.data[i_part][i_mesh_element_type];
        let cols = flat.len() / 3;

        if cols * 3 != flat.len() {
            panic!("Error: vector data is not correctly sized (not divisible by 3)");
        }

        if mesh_cell >= cols {
            panic!("Error: mesh_cell index out of bounds");
        }

        let x = flat[utils::linear_index_coordinates_matrix(mesh_cell, 0)] as f64;
        let y = flat[utils::linear_index_coordinates_matrix(mesh_cell, 1)] as f64;
        let z = flat[utils::linear_index_coordinates_matrix(mesh_cell, 2)] as f64;

        [x, y, z]
    }
}
