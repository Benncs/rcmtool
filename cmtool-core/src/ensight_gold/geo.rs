use crate::{
    ensight_gold::{reader::EnsightGoldReader, types::ElementsType, Reader},
    grid, utils,
};
use std::{
    io::{Error, ErrorKind},
    path::Path,
    str::FromStr,
};

const MAXIMAL_NUMBER_OF_MESH_ELEMENT_TYPE: usize = 20;
const MAXIMAL_NUMBER_OF_PART: usize = 20;

#[derive(Debug, Default, Clone)]
pub(crate) struct MeshElementType {
    pub n_nodes: usize,
    pub(crate) n_elements: usize,
    pub etype: ElementsType,
    pub vertices: Vec<usize>,
}

pub struct Part {
    id: u32,
    name: String,
    vertex_coordinates: Vec<f64>,
    pub n_vertex: usize,
    pub(crate) elements: Vec<MeshElementType>,
}

#[derive(Debug)]
pub struct Geometry {
    pub(crate) parts: Vec<Part>,
}

impl MeshElementType {
    

    pub fn read(reader: &mut EnsightGoldReader, ignore_element_id: bool) -> std::io::Result<Self> {
        let element_type = reader.get_line_string()?;

        if element_type.contains("part") {
            reader.rollback()?;
            return Err(Error::new(ErrorKind::Unsupported, "'part' in header"));
        }

        let etype = ElementsType::from_str(&element_type)
            .map_err(|_| Error::new(ErrorKind::Unsupported, "Missing 'ElementsType' in header"))?;

        let n_elements = reader.read_i32()? as usize;

        if ignore_element_id {
            for _ in 0..n_elements {
                reader.ignore_line()?;
            }
        }
        //Regular
        if etype != ElementsType::Nfaced {
            // let n_vertex = etype.number_vertex();
            let n_nodes = etype.node_count() as usize;
            let mut max = 0;
            let mut vertices = Vec::with_capacity(n_elements * n_nodes);
            for _ in 0..n_elements * n_nodes {
                let cn = reader.read_i32()?;
                vertices.push(cn as usize);
                if cn > max {
                    max = cn;
                }
            }
            println!("{}", max);
            Ok(Self {
                n_nodes,
                n_elements,
                etype,
                vertices,
            })
        } else {
            let n_nodes = etype.node_count() as usize;
            for _ in 0..n_nodes * n_elements {
                println!("{}", reader.read_i32()? as usize);
            }
            todo!()
        }
    }
}

impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part {} with  {} elements", self.id, self.elements.len())
    }
}

impl Part {
    pub fn get_vertex_coordinates(&self, k_vertex: usize, k_xyz: usize) -> f64 {
        self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, k_xyz)]
    }

    pub fn get_vertex_coordinates_vec(&self, k_vertex: usize) -> grid::Coords3 {
        // [
        //     self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, 0)],
        //     self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, 1)],
        //     self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, 2)],
        // ]
        let offset = k_vertex * 3;
        [
            self.vertex_coordinates[offset],
            self.vertex_coordinates[offset + 1],
            self.vertex_coordinates[offset + 2],
        ]
    }

    pub fn set_vertex_coordinates(&mut self, k_vertex: usize, k_xyz: usize, value: f64) {
        self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, k_xyz)] = value;
    }

    pub fn read(
        reader: &mut EnsightGoldReader,
        ignore_node_id: bool,
        ignore_element_id: bool,
    ) -> std::io::Result<Self> {
        let buf = reader.get_line_string()?;

        if !buf.contains("part") {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Missing 'part' in header",
            ));
        }

        let id = reader.read_i32()? as u32;
        reader.ignore_line()?;

        reader.check_lines_contains("coordinates")?;

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
            elements: Vec::with_capacity(MAXIMAL_NUMBER_OF_MESH_ELEMENT_TYPE),
            n_vertex: n_nodes,
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

        while let Ok(element) = MeshElementType::read(reader, ignore_element_id) {
            current_part.elements.push(element);
        }

        Ok(current_part)
    }
}

impl Geometry {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let mut reader = Reader::new(path)?;
        Self::read(&mut reader)
    }

    pub fn number_of_part(&self) -> usize {
        self.parts.len()
    }

    pub fn get_part_by_id(&self, id: u32) -> Option<&Part> {
        self.parts.iter().find(|&p| p.id == id)
    }

    // pub fn get_part_position(&self, part:&Part) -> Option<usize> {
    //     self.parts.iter().position(|p| p.id == part.id)
    // }

    pub fn read(reader: &mut EnsightGoldReader) -> std::io::Result<Self> {
        //SKIP
        for _ in 0..3 {
            reader.ignore_line()?;
        }

        let node_id_choice = reader.get_line_string()?;

        let ignore_node_id = false; //TODO find in node_id_choice: assign

        let element_id_choice = reader.get_line_string()?;
        let ignore_element_id = false; //TODO find in element_id_choice: assign

        println!("{} {}", node_id_choice, element_id_choice);

        let buf = reader.get_line_string()?;
        if buf.contains("extents") {
            println!("IGNORE EXTENTS");
            reader.ignore_bytes(6)?;
        } else {
            reader.rollback()?;
        }

        let mut parts: Vec<Part> = Vec::with_capacity(MAXIMAL_NUMBER_OF_PART);

        while let Ok(part) = Part::read(reader, ignore_node_id, ignore_element_id) {
            parts.push(part);
        }

        if reader.checK_eof()? {
            Ok(Self { parts })
        } else {
            Err(std::io::Error::new(
                ErrorKind::Unsupported,
                "Reader should have been reached EOF".to_string(),
            ))
        }
    }
}
