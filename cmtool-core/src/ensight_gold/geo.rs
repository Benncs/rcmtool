// SPDX-License-Identifier: GPL-3.0-or-later

use crate::coordinates::*;
use crate::{
    ensight_gold::{Reader, reader::EnsightGoldReader, types::ElementsType},
    utils,
};
use std::{
    io::{Error, ErrorKind},
    path::Path,
    str::FromStr,
};

const MAXIMAL_NUMBER_OF_MESH_ELEMENT_TYPE: usize = 20;
const MAXIMAL_NUMBER_OF_PART: usize = 20;

///Marks "the section holds no further item", as opposed to a read failure. Sections are only
///delimited by what the next line holds, so the readers have to report the end as an error and
///the callers must not confuse it with a truncated or unreadable file.
#[derive(Debug)]
struct SectionEnd;

impl std::fmt::Display for SectionEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "end of section")
    }
}

impl std::error::Error for SectionEnd {}

fn section_end() -> Error {
    Error::other(SectionEnd)
}

fn is_section_end(error: &Error) -> bool {
    error
        .get_ref()
        .is_some_and(|inner| inner.is::<SectionEnd>())
}

///The first line of a section is the only place where the file is allowed to end
fn read_section_header(reader: &mut EnsightGoldReader) -> std::io::Result<String> {
    match reader.get_line_string() {
        Ok(line) => Ok(line),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => Err(section_end()),
        Err(error) => Err(error),
    }
}

#[derive(Debug, Default, Clone)]
pub struct FaceElement {
    vertices: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MeshElementType {
    // pub n_nodes: usize,
    //TODO remove: n_nodes is not used and since v0.1.6 reader works for nface and nsided which have no fixed n_nodes
    n_nodes: usize,
    pub(crate) n_elements: usize,
    etype: ElementsType, //n_nodes is inside !
    pub vertices: Vec<usize>,

    pub faces: Option<Vec<FaceElement>>,
}

pub struct Part {
    id: u32,
    name: String,
    vertex_coordinates: Vec<f64>,
    pub n_vertex: usize,
    pub(crate) elements: Vec<MeshElementType>,
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Part #{}: {}", self.id, self.name)?;
        writeln!(f, "  Number of vertices: {}", self.n_vertex)?;
        writeln!(f, "  Vertex coordinates: {}", self.vertex_coordinates.len())?;
        writeln!(f, "  Number of elements: {}", self.elements.len())?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Geometry {
    pub(crate) parts: Vec<Part>,
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Geometry with {} parts", self.parts.len())?;
        for p in &self.parts {
            writeln!(f, "{}", p)?;
        }

        Ok(())
    }
}

impl MeshElementType {
    #[inline]
    pub fn get_type(&self) -> &ElementsType {
        &self.etype
    }

    pub fn get_vertex(&self, i_element: usize, i_vertex: usize) -> usize {
        self.vertices[i_element * self.n_nodes + i_vertex]
    }

    pub fn read(reader: &mut EnsightGoldReader, ignore_element_id: bool) -> std::io::Result<Self> {
        let element_type = read_section_header(reader)?;

        //The next part starts here, this one has no more elements
        if element_type.contains("part") {
            reader.rollback()?;
            return Err(section_end());
        }

        let etype = ElementsType::from_str(&element_type)
            .map_err(|_| Error::new(ErrorKind::Unsupported, "Missing 'ElementsType' in header"))?;

        let n_elements = reader.read_i32()? as usize;

        if !ignore_element_id {
            for _ in 0..n_elements {
                reader.ignore_line()?;
            }
        }

        //TODO: i think condition should be if volume element /else {skip}
        // match volume element like the follwoing if/else if /else

        //Regular
        if etype != ElementsType::VolumeElementType(super::types::VolumeElementTypes::Nfaced)
            && etype != ElementsType::Nsided
        {
            let n_nodes = etype.node_count() as usize;

            let mut vertices = Vec::with_capacity(n_elements * n_nodes);
            for _ in 0..n_elements * n_nodes {
                let cn = reader.read_i32()?;
                vertices.push(cn as usize);
            }

            Ok(Self {
                n_nodes,
                n_elements,
                etype,
                vertices,
                faces: None,
            })
        } else if etype == ElementsType::VolumeElementType(super::types::VolumeElementTypes::Nfaced)
        {
            //Nface defines polygon by their face alowing to have more complexe shape
            let n_nodes = etype.node_count() as usize;

            let mut n_face_per_element = Vec::with_capacity(n_elements);
            let mut n_tot_faces = 0;
            for _ in 0..n_elements {
                let n_face = reader.read_i32()? as usize;
                n_face_per_element.push(n_face);
                n_tot_faces += n_face;
            }

            let faces: Vec<FaceElement> = Vec::new();

            let mut n_tot_nodes = 0;
            let mut n_nodes_pf = Vec::with_capacity(n_tot_faces);
            for i in 0..n_elements {
                for _ in 0..n_face_per_element[i] {
                    let n_nodes_per_face = reader.read_i32()? as usize;
                    n_tot_nodes += n_nodes_per_face;
                    n_nodes_pf.push(n_nodes_per_face);
                }
            }
            let mut vertices = Vec::with_capacity(n_tot_nodes);
            let mut offset = 0;
            for i in 0..n_elements {
                for j in 0..n_face_per_element[i] {
                    let k = offset + j;
                    for _ in 0..n_nodes_pf[k] {
                        let node_number = reader.read_i32()? as usize;
                        vertices.push(node_number);
                    }
                }
                offset += n_face_per_element[i]
            }

            Ok(Self {
                n_nodes,
                n_elements,
                etype,
                vertices,
                faces: Some(faces),
            })
        } else if etype == ElementsType::Nsided {
            // let n_nodes = etype.node_count() as usize;

            let n_nodes = 1;
            let number_of_nodes_per_element = (0..n_elements)
                .map(|_| reader.read_i32())
                .collect::<Result<Vec<_>, _>>()?;
            let n_nodes_tot: i32 = number_of_nodes_per_element.iter().sum();
            let mut vertices = Vec::with_capacity(n_nodes_tot as usize);
            for i in 0..n_elements {
                for _ in 0..number_of_nodes_per_element[i] {
                    let node_number = reader.read_i32()? as usize;
                    vertices.push(node_number);
                }
            }

            Ok(Self {
                n_nodes,
                n_elements,
                etype,
                vertices: vec![],
                faces: None,
            })
        } else {
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

    pub fn get_vertex_coordinates_vec(&self, k_vertex: usize) -> Coords3 {
        let offset = k_vertex * 3;
        [
            self.vertex_coordinates[offset],
            self.vertex_coordinates[offset + 1],
            self.vertex_coordinates[offset + 2],
        ]
    }

    pub fn get_vertex_coordinates_slice(&self, k_vertex: usize) -> &Coords3 {
        let offset = k_vertex * 3;
        self.vertex_coordinates[offset..offset + 3]
            .try_into()
            .expect("Part slice vertex")
    }
    pub fn set_vertex_coordinates(&mut self, k_vertex: usize, k_xyz: usize, value: f64) {
        self.vertex_coordinates[utils::linear_index_coordinates_matrix(k_vertex, k_xyz)] = value;
    }

    pub fn read(
        reader: &mut EnsightGoldReader,
        ignore_node_id: bool,
        ignore_element_id: bool,
    ) -> std::io::Result<Self> {
        let buf = read_section_header(reader)?;

        //Not a part header: the geometry holds no further part
        if !buf.contains("part") {
            return Err(section_end());
        }

        let id = reader.read_i32()? as u32;
        reader.ignore_line()?;

        reader.check_lines_contains("coordinates")?;

        let n_nodes = reader.read_i32()? as usize;

        if !ignore_node_id {
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

        loop {
            match MeshElementType::read(reader, ignore_element_id) {
                Ok(element) => current_part.elements.push(element),
                Err(error) if is_section_end(&error) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(current_part)
    }
}

fn do_ignore(arg: &str) -> bool {
    let parts: Vec<&str> = arg.split(' ').collect();
    if parts.is_empty() {
        panic!("bad args for id choice");
    }
    match *parts.last().unwrap() {
        "assign" | "given" => false,
        "off" | "ignore" => true,
        _ => false,
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

        let (ignore_node_id, ignore_element_id) = {
            let _node_id_choice = reader.get_line_string()?;
            let _element_id_choice = reader.get_line_string()?;

            let ignore_node_id = do_ignore(&_node_id_choice);
            let ignore_element_id = do_ignore(&_node_id_choice);
            (ignore_node_id, ignore_element_id)
        };

        let buf = reader.get_line_string()?;
        if buf.contains("extents") {
            println!("IGNORE EXTENTS");
            reader.ignore_bytes(6)?;
        } else {
            reader.rollback()?;
        }

        let mut parts: Vec<Part> = Vec::with_capacity(MAXIMAL_NUMBER_OF_PART);

        loop {
            match Part::read(reader, ignore_node_id, ignore_element_id) {
                Ok(part) => parts.push(part),
                Err(error) if is_section_end(&error) => break,
                Err(error) => {
                    println!("Error reading part");
                    return Err(error);
                }
            }
        }

        if reader.check_eof()? {
            Ok(Self { parts })
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "{}: trailing data after the last part, {} part(s) read",
                    reader.path().display(),
                    parts.len()
                ),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const LINE_SIZE: usize = 80;
    const N_NODES: i32 = 4;

    fn line(content: &str) -> Vec<u8> {
        let mut buffer = vec![0u8; LINE_SIZE];
        buffer[..content.len()].copy_from_slice(content.as_bytes());
        buffer
    }

    ///One part of four nodes holding a single tetra, the smallest geometry the reader accepts.
    ///Returns the file and the offsets of the coordinate and vertex blocks.
    fn minimal_geometry() -> (Vec<u8>, usize, usize) {
        let mut bytes = Vec::new();

        for _ in 0..3 {
            bytes.extend(line("description"));
        }
        bytes.extend(line("node id off"));
        bytes.extend(line("element id off"));

        //Read once while looking for "extents", then rolled back and read as the part header
        bytes.extend(line("part"));
        bytes.extend(1i32.to_le_bytes());
        bytes.extend(line("a part"));
        bytes.extend(line("coordinates"));
        bytes.extend(N_NODES.to_le_bytes());

        let coordinates_offset = bytes.len();
        for value in 0..3 * N_NODES {
            bytes.extend((value as f32).to_le_bytes());
        }

        bytes.extend(line("tetra4"));
        bytes.extend(1i32.to_le_bytes());

        let vertices_offset = bytes.len();
        for vertex in 0..N_NODES {
            bytes.extend(vertex.to_le_bytes());
        }

        (bytes, coordinates_offset, vertices_offset)
    }

    fn read_geometry(name: &str, bytes: &[u8]) -> std::io::Result<Geometry> {
        let path = std::path::PathBuf::from("/tmp").join(name);
        std::fs::write(&path, bytes).unwrap();
        let geometry = Geometry::new(&path);
        let _ = std::fs::remove_file(&path);
        geometry
    }

    #[test]
    fn test_read_minimal_geometry() {
        let (bytes, _, _) = minimal_geometry();

        let geometry = read_geometry("test_geo_minimal.geo", &bytes).expect("geometry");

        assert_eq!(geometry.number_of_part(), 1);
        assert_eq!(geometry.parts[0].n_vertex, N_NODES as usize);
        assert_eq!(geometry.parts[0].elements.len(), 1);
        assert_eq!(geometry.parts[0].elements[0].n_elements, 1);
    }

    ///A file cut inside the coordinates used to be reported as a geometry without any part
    #[test]
    fn test_truncated_coordinates_is_an_error() {
        let (bytes, coordinates_offset, _) = minimal_geometry();

        let result = read_geometry(
            "test_geo_truncated_coordinates.geo",
            &bytes[..coordinates_offset + 8],
        );

        assert!(result.is_err(), "truncated coordinates read as a geometry");
    }

    ///A file cut inside the element vertices used to yield a part without its elements
    #[test]
    fn test_truncated_elements_is_an_error() {
        let (bytes, _, vertices_offset) = minimal_geometry();

        let result = read_geometry(
            "test_geo_truncated_elements.geo",
            &bytes[..vertices_offset + 4],
        );

        assert!(result.is_err(), "truncated elements read as a geometry");
    }
}
