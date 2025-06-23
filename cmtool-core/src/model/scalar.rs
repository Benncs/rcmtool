use std::ops::Index;

pub struct Scalar
{
    value_in_vo:Vec<cmtool_data::ScalarValueType>
}

impl Scalar
{
    pub fn new()->Self
    {
        Self{value_in_vo:vec![]}
    }
}

impl Index<usize> for Scalar {
    type Output = cmtool_data::ScalarValueType;

    fn index(&self, index: usize) -> &Self::Output {
        &self.value_in_vo[index]
    }
}

// impl IndexMut<usize> for Scalar {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         let (part, mesh_element_type, mesh_cell) = index;
//         let flat_index = part * (self.mesh_element_types * self.mesh_cells)
//             + mesh_element_type * self.mesh_cells
//             + mesh_cell;
//         &mut self.data[flat_index]
//     }
// }