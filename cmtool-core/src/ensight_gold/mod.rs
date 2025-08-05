mod reader;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
};
mod geo;
pub mod types;
pub use crate::{ensight_gold::reader::Reader, utils};
pub mod scalar;
pub mod vectors;
pub use geo::Geometry;
pub use geo::Part;
pub mod case;
pub mod variable;

pub trait RawField: Sized {
    fn init(geometry: Arc<Geometry>, path: impl AsRef<Path>) -> std::io::Result<Self>;
}

// impl super::CfdCase for Case {
//     fn get_root(&self) -> String {
//         self.root.clone()
//     }

//     fn get_geometry_relative_path(&self) -> String {
//         self.geometry_file_path.clone()
//     }
// }

#[cfg(test)]
mod test {
    use crate::ensight_gold::case::{Case, VariableInfo, VariableType};

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
