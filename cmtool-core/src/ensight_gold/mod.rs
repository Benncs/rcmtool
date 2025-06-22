mod reader;
use std::{
    fmt::Debug,
    fs::{read, File},
    io::{BufRead, BufReader, Error, ErrorKind, Lines},
    path::Path,
    str::FromStr,
};
mod geo;
mod types;
use crate::ensight_gold::types::ElementsType;
pub use crate::{ensight_gold::reader::Reader, utils};
pub mod scalar;

pub use geo::Geometry;

#[derive(Default, Debug)]
struct VariableInfo {
    var_type: String,
    name: String,
    filepath: String,
}

impl VariableInfo {
    fn read(line: &str) -> std::io::Result<Self> {
        let mut tokens = line.split_whitespace();
        let mut var_info = VariableInfo::default();

        if let Some(var_type) = tokens.next() {
            var_info.var_type = var_type.to_string();
        }

        if var_info.var_type == "scalar" || var_info.var_type == "vector" {
            tokens.next(); // Skip "per"

            tokens.next(); // Skip "element:"
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
    geometry_file_path: String,
    paths: Vec<VariableInfo>,
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
                    println!("{}", line);
                    case.paths.push(VariableInfo::read(&line)?);
                    line.clear();
                    reader.read_line(&mut line)?;
                }
            }
        }

        Ok(())
    }

    pub fn read(path: &Path) -> std::io::Result<Case> {
        let fd = File::open(path)?;
        let mut buffer = BufReader::new(fd);
        let mut case = Case {
            geometry_file_path: String::new(),
            paths: vec![],
        };

        Self::read_from_buffer(&mut buffer, &mut case)?;

        

        Ok(case)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
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

        // Create a temporary file path
        let mut temp_file_path = env::temp_dir();
        temp_file_path.push("temp_test_file.encas");

        // Write the reference string to the temporary file
        let mut file = File::create(&temp_file_path).expect("Failed to create temporary file");
        write!(file, "{}", reference).expect("Failed to write to temporary file");

        // Read the temporary file using the `Case::read` function
        let case = Case::read(&temp_file_path).expect("Failed to read temporary file");

        // Define the reference `Case` object
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
        };

        // Compare the result to the reference `Case` object
        assert_eq!(case.geometry_file_path, reference_case.geometry_file_path);
        assert_eq!(case.paths.len(), reference_case.paths.len());

        for (i, path) in case.paths.iter().enumerate() {
            let reference_path = &reference_case.paths[i];
            assert_eq!(path.var_type, reference_path.var_type);
            assert_eq!(path.name, reference_path.name);
            assert_eq!(path.filepath, reference_path.filepath);
        }

        // Clean up: remove the temporary file
        std::fs::remove_file(&temp_file_path).expect("Failed to remove temporary file");
    }
}
