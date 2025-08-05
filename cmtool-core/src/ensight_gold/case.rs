use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::CoreError;

#[derive(Clone, Copy)]
pub enum VariableType {
    Scalar,
    Vector,
}

impl TryInto<VariableType> for String {
    type Error = ();

    fn try_into(self) -> Result<VariableType, Self::Error> {
        match self {
            val if val == *"scalar" => Ok(VariableType::Scalar),
            val if val == *"vector" => Ok(VariableType::Vector),
            _ => Err(()),
        }
    }
}

#[derive(Default, Debug)]
pub struct VariableInfo {
    pub var_type: String,
    pub name: String,
    pub filepath: String,
}

impl VariableInfo {
    pub fn get_type(&self) -> VariableType {
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

    pub fn read(path: impl AsRef<Path>) -> Result<Case, CoreError> {
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
                Err(CoreError::Custom("Path is not valid UTF-8".to_owned()))
            }
        } else {
            Err(CoreError::Custom("No parent directory".to_owned()))
        }
    }
}
