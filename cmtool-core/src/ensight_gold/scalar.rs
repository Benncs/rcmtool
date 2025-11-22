// SPDX-License-Identifier: GPL-3.0-or-later

use crate::ensight_gold::{
    reader::EnsightGoldReader,
    variable::{PerElementVariable, VarTypeReader},
};

pub(crate) struct ScalarReader;

impl VarTypeReader for ScalarReader {
    type VarType = Vec<f32>;
    fn read_elements(
        reader: &mut EnsightGoldReader,
        element: &super::geo::MeshElementType,
    ) -> std::io::Result<Self::VarType> {
        reader.read_buffer_f32(element.n_elements)
    }
}
pub(crate) type ScalarField = PerElementVariable<ScalarReader>;

impl ScalarField {
    #[inline]
    pub fn get_value(&self, i_part: usize, i_mesh_element_type: usize, mesh_cell: usize) -> f32 {
        self.data[i_part][i_mesh_element_type][mesh_cell]
    }
}

// impl RawField for ScalarField
// {
//     fn init(geometry: Arc<Geometry>, path: impl AsRef<Path>) -> std::io::Result<Self> {
//         let mut reader = Reader::new(path)?;
//         Self::read(&geometry, &mut reader)
//     }
// }

// impl ScalarField {
//     fn new() -> Self {
//         let data = Vec::new();
//         let part_id = Vec::new();
//         ScalarField { data, part_id,name:String::new() }
//     }

//     pub fn get_value(&self, i_part: usize, i_mesh_element_type: usize, mesh_cell: usize) -> f32 {
//         self.data[i_part][i_mesh_element_type][mesh_cell]
//     }

//     fn per_node_read(geometry: &Geometry, reader: &mut EnsightGoldReader) -> std::io::Result<Self> {
//         reader.ignore_line()?; //description

//         let mut scalar = ScalarField::new();
//         scalar.data.resize(geometry.number_of_part(), Vec::new());

//         for data_in_part in &mut scalar.data {
//             reader.check_lines_contains("part")?;

//             let id = reader.read_i32()?;
//             scalar.part_id.push(id as u32);
//             println!("id {:?}", id);

//             if let Some(part) = geometry.get_part_by_id(id as u32) {
//                 let n_elements = part.elements.len();
//                 println!("{}", n_elements);
//                 data_in_part.push(vec![0.; n_elements]);

//                 let element_type_name = reader.get_line_string()?;
//                 println!("{}", element_type_name);
//                 for (i_element, element) in data_in_part.iter_mut().enumerate() {
//                     println!("{}", i_element);
//                     element.resize(part.elements[i_element].n_elements, 0.);
//                     for element_value in element {
//                         println!("{}", element_value);
//                         *element_value = reader.read_f32()?;
//                     }
//                 }
//             }
//             reader.ignore_line()?; //description
//         }
//         if reader.check_eof()? {
//             Ok(scalar)
//         } else {
//             Err(std::io::Error::new(
//                 ErrorKind::Unsupported,
//                 "Reader should have been reached EOF".to_string(),
//             ))
//         }
//     }

//     fn per_element_read(
//         geometry: &Geometry,
//         reader: &mut EnsightGoldReader,
//     ) -> std::io::Result<Self> {
//         // reader.ignore_line()?; //description
//         let mut scalar = ScalarField::new();
//         scalar.name =reader.get_line_string()?;

//         scalar.data.resize(geometry.number_of_part(), Vec::new());
//         for data_in_part in &mut scalar.data {
//             reader.check_lines_contains("part")?;

//             let id = reader.read_i32()?;
//             scalar.part_id.push(id as u32);

//             if let Some(part) = geometry.get_part_by_id(id as u32) {
//                 let n_elements = part.elements.len();
//                 // println!("{}", n_elements);
//                 data_in_part.reserve(n_elements);

//                 for element in &part.elements {
//                     let element_type_name = reader.get_line_string()?;
//                     if element_type_name.contains("undef")
//                     {
//                         unimplemented!("Undef varaible per element");
//                     }
//                     // let _ = reader.read_f32()?; //undef
//                     // println!("{}", element_type_name);
//                     if ElementsType::from_str(&element_type_name) == Ok(element.etype) {
//                         data_in_part.push(reader.read_buffer_f32(element.n_elements)?);
//                     } else {
//                         return Err(std::io::Error::new(
//                             ErrorKind::Unsupported,
//                             format!(
//                                 "Scalar and geometry part are not the same: {} vs {:?}",
//                                 element_type_name, element.etype
//                             ),
//                         ));
//                     }
//                 }
//             }
//         }
//         if reader.check_eof()? {
//             Ok(scalar)
//         } else {
//             Err(std::io::Error::new(
//                 ErrorKind::Unsupported,
//                 "Reader should have been reached EOF".to_string(),
//             ))
//         }
//     }

//     fn read(geometry: &Geometry, reader: &mut EnsightGoldReader) -> std::io::Result<Self> {
//         // Self::per_node_read(geometry, reader)

//         Self::per_element_read(geometry, reader)
//     }
// }
