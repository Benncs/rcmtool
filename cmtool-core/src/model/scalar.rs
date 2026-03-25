// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::Index;

use cmtool_data::ScalarValueType;

use crate::{
    CoreError,
    ensight_gold::{self, types::ElementsType},
    model::CMGeometry,
};

pub struct Scalar {
    pub(crate) value_in_vo: Vec<cmtool_data::ScalarValueType>,
    pub name: String,
}

impl Scalar {
    pub(crate) fn new(
        eg_scalar: ensight_gold::scalar::ScalarField,
        geometry: &CMGeometry,
        eg_geometry: &ensight_gold::Geometry,
    ) -> Self {
        let mut value_in_vo: Vec<cmtool_data::ScalarValueType> =
            vec![0.; geometry.volume_elements.n_element()];

        for (i_part, part) in eg_geometry.parts.iter().enumerate() {
            for (i_e, element) in part.elements.iter().enumerate() {
                if let ElementsType::VolumeElementType(vetype) = element.etype {
                    let element_index = vetype.to_index();

                    for volume_element_id in 0..element.n_elements {
                        let volume_element_global_id = geometry.volume_elements.get_global_id(
                            i_part,
                            element_index,
                            volume_element_id,
                        );
                        value_in_vo[volume_element_global_id] =
                            eg_scalar.get_value(i_part, i_e, volume_element_id).into();
                    }
                }
            }
        }

        Self {
            value_in_vo,
            name: eg_scalar.get_name().to_string(),
        }
    }

    pub fn element_wise(self, a: &Self) -> Result<Self, CoreError> {
        if a.value_in_vo.len() != self.value_in_vo.len() {
            return Err(CoreError::Custom(format!(
                "Bad size for scalar scaling {} vs {} ",
                self.value_in_vo.len(),
                a.value_in_vo.len()
            )));
        }

        let values: Vec<cmtool_data::ScalarValueType> = a
            .value_in_vo
            .iter()
            .zip(&self.value_in_vo)
            .map(|(a, b)| a * b)
            .collect();
        Ok(Self {
            value_in_vo: values,
            name: format!("{} per {} scalar ", self.name, a.name),
        })
    }

    pub fn scalar_shift(self, lambda: ScalarValueType) -> Self {
        let value_in_vo = self.value_in_vo.iter().map(|i| lambda - i).collect();

        Self {
            value_in_vo,
            name: format!("{} shifted by {} ", self.name, lambda),
        }
    }
}

impl Index<usize> for Scalar {
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
