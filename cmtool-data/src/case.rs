// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{CMAExportType, DataError};
use serde::{Deserialize, Serialize};
use std::ffi::os_str::Display;
use std::io::{BufReader, Read, Write};
use std::{collections::HashMap, fs, path::Path};

/// Represents a case configuration for a computational model analysis.
///
/// Each `CMCase` contains information about the number of divisions, a description of the case,
/// the time spent per flow map, and paths to exported data based on different types.
///
/// # Fields
///
/// * `n_div` - An array of three unsigned integers representing the number of divisions in each dimension.
/// * `description` - A string describing the case configuration.
/// * `time_per_flow_map` - A floating-point value representing the time per flow map in seconds.
/// * `paths` - A map from export types to file paths where the data can be accessed.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CMCase {
    pub n_div: [u32; 3],
    pub description: String,
    pub time_per_flow_map: f64,
    paths: HashMap<CMAExportType, String>,
    pub is_reursive: bool,
}

impl std::fmt::Display for CMCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CMCase Configuration:")?;
        writeln!(
            f,
            "  - Number of Divisions: [{}x{}x{}]",
            self.n_div[0], self.n_div[1], self.n_div[2]
        )?;
        writeln!(f, "  - Description: {}", self.description)?;
        writeln!(
            f,
            "  - Time per Flow Map: {:.2} seconds",
            self.time_per_flow_map
        )?;

        // Show paths, iterating over the HashMap
        writeln!(f, "  - Export Paths:\n")?;
        for export_type in self.paths.keys() {
            writeln!(f, "    - {:?}\n", export_type)?;
        }

        writeln!(
            f,
            "  - Recursive: {}",
            if self.is_reursive { "Yes" } else { "No" }
        )?;
        Ok(())
    }
}

impl CMCase {
    pub fn n_compartment(&self) -> u32 {
        if self.n_div.iter().find(|e| **e == 0).is_some() {
            return 1;
        }
        self.n_div.iter().product()
    }

    pub fn toggle_recursive(&mut self) {
        self.is_reursive = !self.is_reursive;
    }

    pub fn is_two_phase_flow(&self) -> bool {
        self.paths.contains_key(&CMAExportType::GasVolume)
    }

    pub fn add(&mut self, stype: CMAExportType, relative_path: &str) {
        self.paths.insert(stype, relative_path.to_string());
    }

    pub fn resolve(&self, root: &str, stype: CMAExportType) -> Option<String> {
        let rel = self.paths.get(&stype)?;
        Some(Path::new(root).join(rel).to_str()?.to_string())
    }
    pub fn prepend_path(mut self, prep: &str) -> Self {
        for (_key, path) in self.paths.iter_mut() {
            *path = format!("{}/{}", prep, path);
        }
        self
    }

    pub fn get_folders(&self, root: &str) -> Vec<String> {
        let mut folders: Vec<String> = std::fs::read_dir(root)
            .unwrap()
            .filter_map(|entry| {
                if let Ok(dir) = entry {
                    let file_name = dir.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    if let Some(index_str) = file_name_str.strip_prefix("i_") {
                        if index_str.parse::<usize>().is_ok() {
                            Some(file_name_str.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        folders.sort_by(|a, b| {
            let index_a: usize = a.trim_start_matches("i_").parse().unwrap_or(0);
            let index_b: usize = b.trim_start_matches("i_").parse().unwrap_or(0);
            index_a.cmp(&index_b)
        });
        folders
    }

    fn check(&self) -> bool {
        let has_gas_volume = self.paths.contains_key(&CMAExportType::GasVolume);
        let has_gas_flow = self.paths.contains_key(&CMAExportType::GasFlow);

        let has_liq_volume = self.paths.contains_key(&CMAExportType::LiquidVolume);
        let has_liq_flow = self.paths.contains_key(&CMAExportType::LiquidFlow);

        let ok_gas = if has_gas_volume && !has_gas_flow {
            false
        } else {
            !(has_gas_flow && !has_gas_volume)
        };

        let ok_liq = if has_liq_volume && !has_liq_flow {
            false
        } else {
            !(has_liq_flow && !has_liq_volume)
        };

        ok_liq && ok_gas
    }

    pub fn new(
        n_div: [u32; 3],
        time_per_flow_map: f64,
        description: Option<String>,
        recursive: bool,
    ) -> Self {
        let description = description.unwrap_or(String::from("Case"));
        Self {
            n_div,
            time_per_flow_map,
            is_reursive: recursive,
            description,
            paths: HashMap::new(),
        }
    }
}

/// A trait for reading a `CMCase` from a specified path.
///
/// Implement this trait for types that are capable of reading a case configuration
/// from disk or another storage medium.
pub trait CMCaseReader {
    /// Reads a `CMCase` from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the `Path` from which to read the case configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a `CMCase` on success or a `DataError` on failure.
    fn read_case(path: &Path) -> Result<CMCase, DataError>;
}

pub fn read_case(path: &Path) -> Result<CMCase, DataError> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = [0; 4]; // read the first few bytes
    let n = file.read(&mut buffer)?;

    // Simple heuristic: if starts with '{' or '[' treat as JSON
    if n > 0 && (buffer[0] == b'{' || buffer[0] == b'[') {
        CMCaseJson::read_case(path)
    } else {
        CCMCaseInfo::read_case(path)
    }
}

/// A trait for writing a `CMCase` to a specified path.
///
/// Implement this trait for types that are capable of writing a case configuration
/// to disk or another storage medium.
pub trait CMCaseWriter {
    /// Writes a `CMCase` to the given path.
    ///
    /// # Arguments
    ///
    /// * `case` - The `CMCase` instance to write.
    /// * `path` - A reference to the `Path` where the case configuration should be written.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or a `DataError` on failure.
    fn write_case(case: CMCase, path: &Path) -> Result<(), DataError>;
}
/// A type responsible for reading and writing `CMCase` instances to/from JSON files.
pub struct CMCaseJson;

/// A type responsible for reading and writing `CMCase` instances C comparible (binary) files.
pub struct CCMCaseInfo;

impl CMCaseReader for CMCaseJson {
    fn read_case(path: &Path) -> Result<CMCase, DataError> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let case = serde_json::from_str(&contents).map_err(|_| DataError::Serde)?;
        Ok(case)
    }
}

impl CMCaseWriter for CMCaseJson {
    fn write_case(case: CMCase, path: &Path) -> Result<(), DataError> {
        if !case.check() {
            return Err(DataError::BadData);
        }

        let json_string = serde_json::to_string(&case).map_err(|_| DataError::Serde)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(json_string.as_bytes())?;

        Ok(())
    }
}

impl CMCaseReader for CCMCaseInfo {
    fn read_case(path: &Path) -> Result<CMCase, DataError> {
        //C Caseformat do not have recursive flag, manual detection here:

        let root = path.parent().unwrap();
        let is_recursive = if root.is_dir() {
            std::fs::read_dir(root).unwrap().any(|entry| {
                if let Ok(dir) = entry {
                    let file_name = dir.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    return file_name_str.starts_with("i_");
                }
                false
            })
        } else {
            false
        };

        let file = fs::File::open(path)?;
        let mut buffer = BufReader::new(file);

        let mut char_buf = [0u8; 1];
        let mut buf = [0u8; 4];
        let mut buffer_8bytes = [0u8; 8];

        let mut case = CMCase::default();
        if is_recursive {
            case.toggle_recursive();
        }
        for i in &mut case.n_div {
            buffer.read_exact(&mut buf)?;
            *i = u32::from_le_bytes(buf);
        }

        buffer.read_exact(&mut buf)?;
        let string_size = u32::from_le_bytes(buf);

        let mut string_buf = vec![0; string_size as usize];
        buffer.read_exact(&mut string_buf)?;
        case.description = unsafe { String::from_utf8_unchecked(string_buf) };

        buffer.read_exact(&mut buffer_8bytes)?;

        case.time_per_flow_map = f64::from_le_bytes(buffer_8bytes);

        buffer.read_exact(&mut buffer_8bytes)?;
        let map_size = usize::from_le_bytes(buffer_8bytes);

        for _ in 0..map_size {
            buffer.read_exact(&mut char_buf)?;

            let key = CMAExportType::from(i8::from_le_bytes(char_buf));

            buffer.read_exact(&mut buf)?;
            let string_size = u32::from_le_bytes(buf);

            let mut string_buf = vec![0; string_size as usize];
            buffer.read_exact(&mut string_buf)?;
            let value = unsafe { String::from_utf8_unchecked(string_buf) };

            case.paths.insert(key, value);
        }

        Ok(case)
    }
}

impl CMCaseWriter for CCMCaseInfo {
    fn write_case(case: CMCase, path: &Path) -> Result<(), DataError> {
        if !case.check() {
            return Err(DataError::BadData);
        }

        let mut file = std::fs::File::create(path)?;

        for &div in &case.n_div {
            file.write_all(&div.to_le_bytes())?;
        }

        let description_bytes = case.description.as_bytes();
        file.write_all(&(description_bytes.len() as u32).to_le_bytes())?;
        file.write_all(description_bytes)?;

        file.write_all(&case.time_per_flow_map.to_le_bytes())?;

        file.write_all(&(case.paths.len() as u64).to_le_bytes())?;

        for (key, value) in &case.paths {
            file.write_all(&(*key as i8).to_le_bytes())?;
            let value_bytes = value.as_bytes();
            file.write_all(&(value_bytes.len() as u32).to_le_bytes())?;
            file.write_all(value_bytes)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::fs::remove_file;

    use super::*;

    fn commomn_read_test<T: CMCaseReader>() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR"); // compile-time
        let binding = Path::new(manifest_dir).join("test_data/cma_case");
        let path = binding.as_path();

        println!("{:?}", path);

        let rcase = T::read_case(path);

        assert!(rcase.is_ok());

        let case = rcase.unwrap();

        assert!(case.n_div == [6, 6, 12]);
        assert!(case.description == *"Sanofi");
        assert!(case.time_per_flow_map == 0.);
        assert!(case.paths.get(&CMAExportType::LiquidVolume).unwrap() == "./raw/./vofL.raw");

        eprintln!("{:?}", case)
    }

    fn commomn_write_read_test<T: CMCaseReader, F: CMCaseWriter>() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR"); // compile-time
        let binding = Path::new(manifest_dir).join("test_data/cma_case");
        let path = binding.as_path();

        let rcase = T::read_case(path);

        assert!(rcase.is_ok());

        let case = rcase.unwrap();

        assert!(F::write_case(case, Path::new("./case_test")).is_ok());

        let wr_case = T::read_case(Path::new("./case_test"));
        assert!(wr_case.is_ok());
        let wr_case = wr_case.unwrap();
        assert!(wr_case.n_div == [6, 6, 12]);
        assert!(wr_case.description == *"Sanofi");
        assert!(wr_case.time_per_flow_map == 0.);
        assert!(wr_case.paths.get(&CMAExportType::LiquidVolume).unwrap() == "./raw/./vofL.raw");

        std::fs::remove_file("./case_test").unwrap();
    }

    #[test]
    fn test_read_c_compatible() {
        commomn_read_test::<CCMCaseInfo>();
    }

    #[test]
    fn test_read_write_c_compatible() {
        commomn_write_read_test::<CCMCaseInfo, CCMCaseInfo>();
    }

    #[test]
    fn test_conversion() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR"); // compile-time
        let binding = Path::new(manifest_dir).join("test_data/cma_case");
        let c_path = binding.as_path();

        let reference_case = CCMCaseInfo::read_case(c_path).unwrap();

        let rcase = CCMCaseInfo::read_case(c_path).unwrap();

        CMCaseJson::write_case(rcase, Path::new("test_2case.json")).unwrap();

        let converted = CMCaseJson::read_case(Path::new("test_2case.json")).unwrap();

        assert!(reference_case.n_div == converted.n_div);
        assert!(reference_case.description == converted.description);
        assert!(reference_case.time_per_flow_map == converted.time_per_flow_map);
        assert!(reference_case.paths == converted.paths);

        remove_file(Path::new("test_2case.json")).expect("Failed to remove test file")
    }

    fn common_write_read_test<T: CMCaseWriter + CMCaseReader>(path: &Path) -> Result<(), ()> {
        let case = CMCase {
            n_div: [4, 5, 1],
            description: "Test".to_string(),
            time_per_flow_map: 0.01,
            paths: HashMap::new(),
            is_reursive: false,
        };

        T::write_case(case, path).map_err(|_| ())?;
        let read_case = T::read_case(path).map_err(|_| ())?;
        assert!(read_case.n_div == [4, 5, 1]);
        assert!(read_case.description == *"Test");
        assert!(read_case.time_per_flow_map == 0.01);
        Ok(())
    }

    #[test]
    fn test_read_json() {
        let path = Path::new("test_case.json");
        let case = CMCase {
            n_div: [4, 5, 1],
            description: "Test".to_string(),
            time_per_flow_map: 0.01,
            paths: HashMap::new(),
            is_reursive: false,
        };

        CMCaseJson::write_case(case, path).expect("Failed to write case");
        let read_case = CMCaseJson::read_case(path).expect("Failed to read case");
        assert!(read_case.n_div == [4, 5, 1]);
        assert!(read_case.description == *"Test");
        assert!(read_case.time_per_flow_map == 0.01);

        remove_file(path).expect("Failed to remove test file");
    }

    #[test]
    fn test_read_write() {
        let path = Path::new("test_case_common.json");
        common_write_read_test::<CMCaseJson>(path).expect("Common write-read test failed");
        remove_file(path).expect("Failed to remove test file");
    }
}
