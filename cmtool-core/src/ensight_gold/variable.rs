use std::{io::ErrorKind, path::Path, str::FromStr, sync::Arc};

use crate::ensight_gold::{
    geo::{Geometry, MeshElementType}, reader::EnsightGoldReader, types::ElementsType, Reader,
};

pub(crate) trait VarTypeReader {
    type VarType:Clone;
    fn read_elements(
        reader: &mut EnsightGoldReader,
        element: &MeshElementType,
    ) -> std::io::Result<Self::VarType>;
}

#[derive(Debug)]
pub(crate) struct PerElementVariable<T:VarTypeReader> {
    // data: Vec<f32>,
    // parts: usize,
    // mesh_element_types: usize,
    // mesh_cells: usize,
    name: String,
    pub part_id: Vec<u32>,
    pub(super) data: Vec<Vec<T::VarType>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T:VarTypeReader> PerElementVariable<T> {
    fn new() -> Self {
        let data = Vec::new();
        let part_id = Vec::new();
        Self {
            data,
            part_id,
            name: String::new(),
            _marker: std::marker::PhantomData,
        
        }
    }
    pub fn get_name(&self)->&str
    {
        &self.name
    }
    pub fn init(geometry: Arc<Geometry>, path: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut reader = Reader::new(path)?;
        Self::read(&geometry, &mut reader)
    }

    fn read(geometry: &Geometry, reader: &mut EnsightGoldReader) -> std::io::Result<Self> {
        // reader.ignore_line()?; //description
        let mut variable = Self::new();
        variable.name = reader.get_line_string()?;

        variable.data.resize(geometry.number_of_part(), Vec::new());
        for data_in_part in &mut variable.data {
            reader.check_lines_contains("part")?;

            let id = reader.read_i32()?;
            variable.part_id.push(id as u32);

            if let Some(part) = geometry.get_part_by_id(id as u32) {
                let n_elements = part.elements.len();
                // println!("{}", n_elements);
                data_in_part.reserve(n_elements);

                for element in &part.elements {
                    let element_type_name = reader.get_line_string()?;
                    if element_type_name.contains("undef") {
                        unimplemented!("Undef varaible per element");
                    }
                    // let _ = reader.read_f32()?; //undef
                    // println!("{}", element_type_name);
                    if ElementsType::from_str(&element_type_name) == Ok(element.etype) {
                        data_in_part.push(T::read_elements(reader, element)?);
                    } else {
                        return Err(std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Scalar and geometry part are not the same: {} vs {:?}",
                                element_type_name, element.etype
                            ),
                        ));
                    }
                }
            }
        }
        if reader.check_eof()? {
            Ok(variable)
        } else {
            Err(std::io::Error::new(
                ErrorKind::Unsupported,
                "Reader should have been reached EOF".to_string(),
            ))
        }
    }
}
