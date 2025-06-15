mod reader;
use std::{
    fmt::Debug, fs::read, io::{Error, ErrorKind}, path::Path, str::FromStr
};
mod types;
use crate::ensight_gold::types::ElementsType;
pub use crate::{ensight_gold::reader::Reader, utils};

const MaximalNumberOfMeshElementType: usize = 20;
const MaximalNumberOfPart: usize = 20;

#[derive(Debug,Default, Copy, Clone)]
struct MeshElement {}

impl MeshElement {
    pub fn read(reader: &mut Reader, ignore_element_id: bool) -> std::io::Result<Self> {

        let element_type = reader.get_line()?.to_string();

        if element_type.contains("part") {
            reader.rollback()?;
            return Err(Error::new(
                ErrorKind::Unsupported,
                "'part' in header",
            ));
        }

        let etype = ElementsType::from_str(&element_type.to_string())
            .map_err(|_| Error::new(ErrorKind::Unsupported, "Missing 'ElementsType' in header"))?;

        let n_elements = reader.read_i32()? as usize;

        if ignore_element_id {
            for _ in 0..n_elements {
                reader.ignore_line()?;
            }
        }
        //Regular
        if etype != ElementsType::Nfaced {
            let n_vertex = etype.number_vertex();
            let n_nodes = etype.number_of_nodes() as usize;
            let mut max = 0;
            for i_e in 0..n_elements * n_nodes {
                let cn = reader.read_i32()?;
                if cn > max {
                    max = cn;
                }
            }
            println!("{}", max);
        } else {
            let n_nodes = etype.number_of_nodes() as usize;
            for i in 0..n_nodes * n_elements {
                println!("{}", reader.read_i32()? as usize);
            }
        }
        Ok(Self {})
    }
}


struct Part {
    id: u32,
    name: String,
    vertex_coordinates: Vec<f64>,
    elements: Vec<MeshElement>,
}

impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part {} with  {} elements", self.id,self.elements.len())
    }
}


impl Part {
    pub fn get_vertex_coordinates(&self, k_vertex: usize, k_xyz: usize) -> f64 {
        self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, k_xyz)]
    }

    pub fn set_vertex_coordinates(&mut self, k_vertex: usize, k_xyz: usize, value: f64) {
        self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, k_xyz)] = value;
    }

    pub fn read(
        reader: &mut Reader,
        ignore_node_id: bool,
        ignore_element_id: bool,
    ) -> std::io::Result<Self> {
        let mut buf = reader.get_line()?;

        if !buf.to_string().contains("part") {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Missing 'part' in header",
            ));
        }

        let id = reader.read_i32()? as u32;
        reader.ignore_line()?;

        buf = reader.get_line()?;

        if !buf.to_string().contains("coordinates") {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Missing 'coordinates' in header",
            ));
        }

        let n_nodes = reader.read_i32()? as usize;

        if ignore_node_id {
            for _ in 0..n_nodes {
                reader.ignore_line()?;
            }
        }

        let mut current_part = Self {
            id,
            name: "part".to_string(),
            vertex_coordinates: vec![0.; n_nodes * 3],
            elements: Vec::with_capacity(MaximalNumberOfMeshElementType),
        };
        for i_xyz in 0..3 {
            for i_vertex in 0..n_nodes {
                current_part.vertex_coordinates
                    [utils::linear_index_coordinates_matrix(i_vertex, i_xyz)] =
                    reader.read_f32()? as f64;
            }
        }

        println!("Loaded Part {}", id);
        println!("n_nodes {}", n_nodes);

        while let Ok(element) = MeshElement::read(reader, ignore_element_id) {
            current_part.elements.push(element);
        }

        Ok(current_part)
    }
}


#[derive(Debug)]
pub struct Geometry {
    parts: Vec<Part>,
}



impl Geometry {
    pub fn read(reader: &mut Reader) -> std::io::Result<Self> {
        //SKIP
        for i in 0..3 {
            reader.ignore_line();
        }

        let node_id_choice = reader.get_line()?.to_string();

        let ignore_node_id = false; //TODO find in node_id_choice: assign

        let element_id_choice = reader.get_line()?.to_string();
        let ignore_element_id = false; //TODO find in element_id_choice: assign

        println!("{} {}", node_id_choice, element_id_choice);

        let buf = reader.get_line()?;
        if buf.to_string().contains("extents") {
            println!("IGNORE EXTENTS");
            reader.ignore_bytes(6)?;
        } else {
            reader.rollback()?;
        }

        let mut parts : Vec<Part> = Vec::with_capacity(MaximalNumberOfPart);

        while let Ok(part) = Part::read(reader, ignore_node_id, ignore_element_id) {
            parts.push(part);
        }

        Ok(Self{parts})
    }
}
