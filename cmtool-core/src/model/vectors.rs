use std::ops::Index;

use crate::{
    ensight_gold::{self, types::ElementsType},
    model::CMGeometry,
    CoreError,
};

pub struct Vector {
    value_in_vo: Vec<cmtool_data::ScalarValueType>,
}

impl Vector {
    pub fn get_slice_xyz(&self, global_id: usize) -> &[f64; 3] {
        let offset = global_id * 3;
        self.value_in_vo[offset..offset + 3]
            .try_into()
            .expect("Slice with exactly 3 elements")
    }
    pub(crate) fn from_scalar(
        scalars: [ensight_gold::scalar::ScalarField; 3],
        geometry: &CMGeometry,
        eg_geometry: &ensight_gold::Geometry,
    ) -> Result<Self, CoreError> {
        let functor = |i_part, i_e, volume_element_id| {
            [
                scalars[0].get_value(i_part, i_e, volume_element_id) as f64,
                scalars[1].get_value(i_part, i_e, volume_element_id) as f64,
                scalars[2].get_value(i_part, i_e, volume_element_id) as f64,
            ]
        };
        Ok(Self::from_xyz(geometry, eg_geometry, functor))
    }

    fn from_xyz(
        geometry: &CMGeometry,
        eg_geometry: &ensight_gold::Geometry,
        f: impl Fn(usize, usize, usize) -> [f64; 3],
    ) -> Self {
        let mut value_in_vo: Vec<cmtool_data::ScalarValueType> =
            vec![0.; 3 * geometry.volume_elements.n_element()];
        for (i_part, part) in eg_geometry.parts.iter().enumerate() {
            for (i_e, element) in part.elements.iter().enumerate() {
                match element.etype {
                    ElementsType::VolumeElementType(vetype) => {
                        //Unwrap never fails because "get_part_by_id" has already identified part

                        let element_index = vetype.to_index();

                        for volume_element_id in 0..element.n_elements {
                            let volume_element_global_id = geometry.volume_elements.get_global_id(
                                i_part,
                                element_index,
                                volume_element_id,
                            );
                            let offset = 3 * volume_element_global_id;
                            value_in_vo[offset..offset + 3].copy_from_slice(&f(
                                i_part,
                                i_e,
                                volume_element_id,
                            ));
                        }
                    }
                    _ => {
                        continue;
                        // unimplemented!("Not volumic element type")
                    }
                }
            }
        }

        Self { value_in_vo }
    }

    ////

    pub(crate) fn new(
        eg_vector: ensight_gold::vectors::VectorField,
        geometry: &CMGeometry,
        eg_geometry: &ensight_gold::Geometry,
    ) -> Self {
        let functor =
            |i_part, i_e, volume_element_id| eg_vector.get_xyz(i_part, i_e, volume_element_id);
        Self::from_xyz(geometry, eg_geometry, functor)
    }
}

impl Index<usize> for Vector {
    type Output = cmtool_data::ScalarValueType;

    fn index(&self, index: usize) -> &Self::Output {
        #[cfg(debug_assertions)]
        {
            // Debug mode: safe indexing with bounds check
            &self.value_in_vo[index]
        }

        #[cfg(not(debug_assertions))]
        unsafe {
            // Release mode: unchecked access (unsafe but fast)
            self.value_in_vo.get_unchecked(index)
        }
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
