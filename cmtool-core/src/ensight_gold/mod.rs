mod reader;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
mod geo;
pub mod types;
pub use crate::{ensight_gold::reader::Reader, utils};
pub mod scalar;

pub use geo::Geometry;
pub use geo::Part;

#[derive(Clone, Copy)]
pub enum VariableType
{
    Scalar,
    Vector
}

// impl From<String> for VariableType
// {
//     fn from(value: String) -> Self {
//         match value
//         {
//             "scalar"=>VariableType::Scalar,
//             "vector"=>VariableType::Vector,
//             _ => 
//         }
//     }
// }

impl TryInto<VariableType> for String
{
    type Error=();

    fn try_into(self) -> Result<VariableType, Self::Error> {
        match self
        {
            val if val == "scalar".to_owned()=>Ok(VariableType::Scalar),
            val if val == "vector".to_owned()=>Ok(VariableType::Vector),
            _ => Err(())
        }
    }
}

#[derive(Default, Debug)]
pub struct VariableInfo {
    var_type: String,
    pub name: String,
    pub filepath: String,
}

impl VariableInfo {

    pub fn get_type(&self)->VariableType
    {
        self.var_type.clone().try_into().unwrap()
    }

    fn read(line: &str) -> std::io::Result<Self> {
        let mut tokens = line.split_whitespace();
        let mut var_info = VariableInfo::default();
        

        if let Some(var_type) = tokens.next() {
            var_info.var_type = var_type.to_string();
        }

        

        if var_info.var_type == "scalar" || var_info.var_type == "vector" {
            tokens.next().expect("Error reading case"); // Skip "per"

            if tokens.next().expect("Error reading case") != "element:" {
                unimplemented!("Eg case Vector/Scalar: per node");
            }
        }

        if let Some(name) = tokens.next() {
            var_info.name = name.trim_matches('"').to_string();
        }

        if let Some(filepath) = tokens.next() {
            var_info.filepath = filepath.trim_matches('"').to_string();
        }

        Ok(var_info)
    }
}

#[derive(Debug)]
pub struct Case {
    pub geometry_file_path: String,
    pub paths: Vec<VariableInfo>,
    pub root: String,
}

impl Case {
    fn read_from_buffer<R: std::io::Read>(
        reader: &mut BufReader<R>,
        case: &mut Case,
    ) -> std::io::Result<()> {
        let mut line = String::new();

        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }

            if line.contains("GEOMETRY") {
                let mut next_line = String::new();
                reader.read_line(&mut next_line)?;
                let mut parts = next_line.split_whitespace();
                parts.next(); // Skip "model:"
                if let Some(path) = parts.next() {
                    case.geometry_file_path = path.trim_matches('"').to_string();
                }
            }

            if line.contains("VARIABLE") {
                line.clear();
                reader.read_line(&mut line)?;

                while !(line.contains("SCRIPTS")
                    || line.contains("MATERIAL")
                    || line.contains("FILE")
                    || line.contains("TIME"))
                {
                    case.paths.push(VariableInfo::read(&line)?);
                    line.clear();
                    if reader.read_line(&mut line)? == 0 {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn read(path: impl AsRef<Path>) -> std::io::Result<Case> {
        if let Some(root_path) = path.as_ref().parent() {
            if let Some(root_str) = root_path.to_str() {
                let root = root_str.to_string();
                let mut case = Case {
                    geometry_file_path: String::new(),
                    paths: vec![],
                    root,
                };
                let fd = File::open(path)?;
                let mut buffer = BufReader::new(fd);

                Self::read_from_buffer(&mut buffer, &mut case)?;

                Ok(case)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Path is not valid UTF-8",
                ))
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No parent directory",
            ))
        }
    }
}

impl super::CfdCase for Case {
    fn get_root(&self) -> String {
        self.root.clone()
    }

    fn get_geometry_relative_path(&self) -> String {
        self.geometry_file_path.clone()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    #[test]
    fn test_read() {
        let reference = "FORMAT
type: ensight gold
GEOMETRY
model: test.geo
VARIABLE
scalar per element: velocity_magnitude test.scl1
scalar per element: x_velocity test.scl2
scalar per element: y_velocity test.scl3
scalar per element: z_velocity test.scl4
scalar per element: turb_kinetic_energy test.scl5
vector per element: velocity test.vel
SCRIPTS
metadata: \"test.xml\"
";

        let mut temp_file_path = env::temp_dir();
        temp_file_path.push("temp_test_file.encas");

        let mut file = File::create(&temp_file_path).expect("Failed to create temporary file");
        write!(file, "{}", reference).expect("Failed to write to temporary file");

        let case = Case::read(&temp_file_path).expect("Failed to read temporary file");

        let reference_case = Case {
            geometry_file_path: "test.geo".to_string(),
            paths: vec![
                VariableInfo {
                    var_type: "scalar".to_string(),
                    name: "velocity_magnitude".to_string(),
                    filepath: "test.scl1".to_string(),
                },
                VariableInfo {
                    var_type: "scalar".to_string(),
                    name: "x_velocity".to_string(),
                    filepath: "test.scl2".to_string(),
                },
                VariableInfo {
                    var_type: "scalar".to_string(),
                    name: "y_velocity".to_string(),
                    filepath: "test.scl3".to_string(),
                },
                VariableInfo {
                    var_type: "scalar".to_string(),
                    name: "z_velocity".to_string(),
                    filepath: "test.scl4".to_string(),
                },
                VariableInfo {
                    var_type: "scalar".to_string(),
                    name: "turb_kinetic_energy".to_string(),
                    filepath: "test.scl5".to_string(),
                },
                VariableInfo {
                    var_type: "vector".to_string(),
                    name: "velocity".to_string(),
                    filepath: "test.vel".to_string(),
                },
            ],
            root: String::new(),
        };

        assert_eq!(case.geometry_file_path, reference_case.geometry_file_path);
        assert_eq!(case.paths.len(), reference_case.paths.len());

        for (i, path) in case.paths.iter().enumerate() {
            let reference_path = &reference_case.paths[i];
            assert_eq!(path.var_type, reference_path.var_type);
            assert_eq!(path.name, reference_path.name);
            assert_eq!(path.filepath, reference_path.filepath);
        }
        std::fs::remove_file(&temp_file_path).expect("Failed to remove temporary file");
    }
}
