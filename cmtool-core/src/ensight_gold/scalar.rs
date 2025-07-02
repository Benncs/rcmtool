use std::{io::ErrorKind, path::Path, sync::Arc};

use crate::ensight_gold::{geo::Geometry, reader::EnsightGoldReader, Reader};

#[derive(Debug)]
pub struct ScalarField {
    // data: Vec<f32>,
    // parts: usize,
    // mesh_element_types: usize,
    // mesh_cells: usize,
    data: Vec<Vec<Vec<f32>>>,
}

// impl Index<(usize, usize, usize)> for ScalarField {
//     type Output = f32;

//     fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
//         let (part, mesh_element_type, mesh_cell) = index;
//         let flat_index = part * (self.mesh_element_types * self.mesh_cells)
//             + mesh_element_type * self.mesh_cells
//             + mesh_cell;
//         &self.data[flat_index]
//     }
// }

// impl IndexMut<(usize, usize, usize)> for ScalarField {
//     fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
//         let (part, mesh_element_type, mesh_cell) = index;
//         let flat_index = part * (self.mesh_element_types * self.mesh_cells)
//             + mesh_element_type * self.mesh_cells
//             + mesh_cell;
//         &mut self.data[flat_index]
//     }
// }

impl ScalarField {
    fn new() -> Self {
        let data = Vec::new();
        ScalarField { data }
    }

    pub fn init(geometry: Arc<Geometry>, path: &Path) -> std::io::Result<Self> {
        let mut reader = Reader::new(path)?;
        Self::read(&geometry, &mut reader)
    }

    fn read(geometry: &Arc<Geometry>, reader: &mut EnsightGoldReader) -> std::io::Result<Self> {
        reader.ignore_line()?; //description

        let mut scalar = ScalarField::new();
        scalar.data.resize(geometry.number_of_part(), Vec::new());

        for i_part in &mut scalar.data {
            reader.check_lines_contains("part")?;

            let id = reader.read_i32()?;
            println!("{:?}", id);

            if let Some(part) = geometry.get_part_by_id(id as u32) {
                let n_elements = part.elements.len();
                i_part.push(Vec::with_capacity(n_elements));

                let element_type_name = reader.get_line_string()?;
                println!("{}", element_type_name);
                for (i_element, element) in i_part.iter_mut().enumerate() {
                    element.resize(part.elements[i_element].n_elements, 0.);
                    for element_value in element {
                        *element_value = reader.read_f32()?;
                    }
                }
            }
        }
        if reader.checK_eof()? {
            Ok(scalar)
        } else {
            Err(std::io::Error::new(
                ErrorKind::Unsupported,
                "Reader should have been reached EOF".to_string(),
            ))
        }
    }
}
