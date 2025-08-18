// SPDX-License-Identifier: GPL-3.0-or-later

use crate::descriptors::{CMExportType, PhaseCM};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub type ScalarValueType = f64;
use crate::DataError;

// A trait for reading and writing raw data to and from storage.
pub trait RawData: Sized {
    /// Attempts to read raw data from a specified path and instantiate an object.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a path from which to read the raw data. It can be any type
    ///   that implements `AsRef<Path>`, such as `String` or `Path`.
    ///
    /// # Returns
    ///
    /// Returns an `Option<Self>`, which will be `Some(Self)` if reading and parsing are successful,
    /// or `None` if an error occurs or the data is not available.
    fn read_raw(path: impl AsRef<Path>) -> Option<Self>;

    /// Attempts to write the raw data to a specified path.
    ///
    /// # Arguments
    ///
    /// * `&self` - The instance of the type implementing `RawData`.
    /// * `path` - A string slice specifying the path where the raw data will be written.
    ///
    /// # Returns
    ///
    /// Returns a `Result<(), DataError>`, indicating success with `Ok(())` or an error
    /// of type `DataError` if writing fails.
    fn write_raw(&self, path: &str) -> Result<(), DataError>;
}

/// Represents the header of a flux file.
///
/// This header contains metadata about the flux data stored in the file,
/// including the number of zones and the number of flux interactions.
#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub struct FluxFileHeader {
    /// The number of zones in the flux file data.
    pub n_zone: u32,
    /// The number of flux interactions in the file.
    pub n_fluxes: u32,
}

/// Represents a single raw flux interaction.
///
/// This struct is used to store individual flux interactions between a source and a target,
/// including the flux values in both directions.
#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct RawFlux {
    /// The identifier for the source in the flux interaction.
    pub id_source: u32,
    /// The identifier for the target in the flux interaction.
    pub id_target: u32,
    /// The flux value from the source to the target.
    pub flux_source_target: f64,
    /// The flux value from the target to the source.
    pub flux_target_source: f64,
}

/// Represents a collection of raw flux data along with its header.
///
/// The `RawDataFlux` struct combines metadata about the flux data (via `FluxFileHeader`)
/// with a vector of `RawFlux` instances, representing the actual flux interactions.
#[derive(Deserialize, Serialize, Clone)]
pub struct RawDataFlux {
    /// The header containing metadata about the flux data.
    pub header: FluxFileHeader,
    /// A vector of `RawFlux` interactions, representing the actual flux data.
    pub fluxes: Vec<RawFlux>,
}

/// Represents the header of a scalar file.
///
/// This header contains metadata about the scalar data stored in the file.
#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub struct ScalarFileHeader {
    /// The number of zones in the scalar file data.
    pub n_zone: u32,
}

/// Represents a single raw scalar value.
///
/// This struct is used to store individual floating-point scalar values.
#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct RawScalar {
    /// The scalar value stored as a floating-point number.
    pub value: f64,
}

/// Represents a collection of raw scalar data along with its header.
///
/// The `RawDataScalar` struct combines metadata about the data (via `ScalarFileHeader`)
/// with a vector of `RawScalar` instances, representing the actual scalar values.
#[derive(Deserialize, Serialize, Clone)]
pub struct RawDataScalar {
    /// The header containing metadata about the scalar data.
    pub header: ScalarFileHeader,
    /// A vector of `RawScalar` values, representing the actual scalar data.
    pub values: Vec<RawScalar>,
}

/// Represents a phase in a multi-phase process or system.
///
/// The `RawPhase` struct is likely used to capture the state or characteristics
/// of a specific phase, including flow data, volume information, and an identifier
/// for the phase itself.
#[derive(Deserialize, Serialize, Clone)]
pub struct RawPhase {
    /// Flow data associated with this phase.
    pub flow: RawDataFlux,

    /// Volume data associated with this phase.
    pub volume: RawDataScalar,

    /// Identifier for this phase.
    pub identifier: PhaseCM,
}

impl RawPhase {
    pub fn new_liquid(n_zone: usize, n_fluxes: usize) -> Self {
        Self::new(n_zone, n_fluxes, PhaseCM::Liquid)
    }
    pub fn new_gas(n_zone: usize, n_fluxes: usize) -> Self {
        Self::new(n_zone, n_fluxes, PhaseCM::Gas)
    }

    pub fn new(n_zone: usize, n_fluxes: usize, phase: PhaseCM) -> Self {
        Self {
            flow: RawDataFlux::new(n_zone, n_fluxes),
            volume: RawDataScalar::new(n_zone),
            identifier: phase,
        }
    }

    pub fn write(&self, root: impl AsRef<std::path::Path>) -> Result<(String, String), DataError> {
        let path = PathBuf::from(root.as_ref())
            .join(CMExportType::Flow(self.identifier).default_filename());
        self.flow.write_raw(path.to_str().unwrap())?;

        let path = PathBuf::from(root.as_ref())
            .join(CMExportType::Volume(self.identifier).default_filename());
        self.volume.write_raw(path.to_str().unwrap())?;

        Ok((
            CMExportType::Flow(self.identifier).default_filename(),
            CMExportType::Volume(self.identifier).default_filename(),
        ))
    }
}

impl Default for RawFlux {
    fn default() -> Self {
        Self {
            id_source: 0,
            id_target: 0,
            flux_source_target: 0.,
            flux_target_source: 0.,
        }
    }
}

impl From<f64> for RawScalar {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Self { value }
    }
}

impl RawDataScalar {
    pub fn new(n_zone: usize) -> Self {
        Self {
            header: ScalarFileHeader {
                n_zone: n_zone.try_into().unwrap(),
            },
            values: Vec::with_capacity(n_zone),
        }
    }
}

impl From<Vec<f64>> for RawDataScalar {
    fn from(value: Vec<f64>) -> Self {
        Self {
            header: ScalarFileHeader {
                n_zone: value.len().try_into().unwrap(),
            },
            values: value.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl RawDataFlux {
    pub fn new(n_zone: usize, n_fluxes: usize) -> Self {
        Self {
            header: FluxFileHeader {
                n_zone: n_zone as u32,
                n_fluxes: n_fluxes as u32,
            },
            fluxes: vec![RawFlux::default(); n_fluxes],
        }
    }
}

impl RawData for RawDataScalar {
    fn read_raw(path: impl AsRef<std::path::Path>) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;

        let mut offset = 0;
        let header = ScalarFileHeader::from_bytes(&buffer, &mut offset)?;

        let mut values = Vec::new();
        while offset < buffer.len() {
            values.push(RawScalar::from_bytes(&buffer, &mut offset)?);
        }

        Some(RawDataScalar { header, values })
    }

    fn write_raw(&self, path: &str) -> Result<(), DataError> {
        let mut file = File::create(Path::new(path))?;
        let mut buffer = Vec::new();

        self.header.to_bytes(&mut buffer);
        for value in &self.values {
            value.to_bytes(&mut buffer);
        }

        file.write_all(&buffer)?;
        Ok(())
    }
}

impl RawData for RawDataFlux {
    fn read_raw(path: impl AsRef<std::path::Path>) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;

        let mut offset = 0;
        let header = FluxFileHeader::from_bytes(&buffer, &mut offset)?;

        let mut fluxes = Vec::new();
        while offset < buffer.len() {
            fluxes.push(RawFlux::from_bytes(&buffer, &mut offset)?);
        }

        Some(RawDataFlux { header, fluxes })
    }

    fn write_raw(&self, path: &str) -> Result<(), DataError> {
        let mut file = File::create(Path::new(path))?;
        let mut buffer = Vec::new();
        self.header.to_bytes(&mut buffer);
        for flux in &self.fluxes {
            flux.to_bytes(&mut buffer);
        }

        file.write_all(&buffer)?;
        Ok(())
    }
}
pub trait FromBytes: Sized {
    fn from_bytes(buffer: &[u8], offset: &mut usize) -> Option<Self>;
}

pub trait ToBytes {
    fn to_bytes(&self, buffer: &mut Vec<u8>);
}

impl FromBytes for ScalarFileHeader {
    fn from_bytes(buffer: &[u8], offset: &mut usize) -> Option<Self> {
        if *offset + size_of::<u32>() > buffer.len() {
            return None;
        }
        let n_zone = u32::from_le_bytes(
            buffer[*offset..*offset + size_of::<u32>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<u32>();
        Some(ScalarFileHeader { n_zone })
    }
}

impl ToBytes for ScalarFileHeader {
    fn to_bytes(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.n_zone.to_le_bytes());
    }
}

impl FromBytes for FluxFileHeader {
    fn from_bytes(buffer: &[u8], offset: &mut usize) -> Option<Self> {
        if *offset + 2 * size_of::<u32>() > buffer.len() {
            return None;
        }
        let n_zone = u32::from_le_bytes(
            buffer[*offset..*offset + size_of::<u32>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<u32>();
        let n_max = u32::from_le_bytes(
            buffer[*offset..*offset + size_of::<u32>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<u32>();
        Some(FluxFileHeader {
            n_zone,
            n_fluxes: n_max,
        })
    }
}

impl ToBytes for FluxFileHeader {
    fn to_bytes(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.n_zone.to_le_bytes());
        buffer.extend_from_slice(&self.n_fluxes.to_le_bytes());
    }
}

impl FromBytes for RawScalar {
    fn from_bytes(buffer: &[u8], offset: &mut usize) -> Option<Self> {
        if *offset + size_of::<f64>() > buffer.len() {
            return None;
        }
        let value = f64::from_le_bytes(
            buffer[*offset..*offset + size_of::<f64>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<f64>();
        Some(RawScalar { value })
    }
}

impl ToBytes for RawScalar {
    fn to_bytes(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.value.to_le_bytes());
    }
}

impl FromBytes for RawFlux {
    fn from_bytes(buffer: &[u8], offset: &mut usize) -> Option<Self> {
        if *offset + 2 * size_of::<u32>() + 2 * size_of::<f64>() > buffer.len() {
            return None;
        }
        let id_source = u32::from_le_bytes(
            buffer[*offset..*offset + size_of::<u32>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<u32>();
        let id_target = u32::from_le_bytes(
            buffer[*offset..*offset + size_of::<u32>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<u32>();
        let flux_source_target = f64::from_le_bytes(
            buffer[*offset..*offset + size_of::<f64>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<f64>();
        let flux_target_source = f64::from_le_bytes(
            buffer[*offset..*offset + size_of::<f64>()]
                .try_into()
                .unwrap(),
        );
        *offset += size_of::<f64>();
        Some(RawFlux {
            id_source,
            id_target,
            flux_source_target,
            flux_target_source,
        })
    }
}

impl ToBytes for RawFlux {
    fn to_bytes(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.id_source.to_le_bytes());
        buffer.extend_from_slice(&self.id_target.to_le_bytes());
        buffer.extend_from_slice(&self.flux_source_target.to_le_bytes());
        buffer.extend_from_slice(&self.flux_target_source.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use crate::rawdata::{FromBytes, RawData, ToBytes};

    use super::*;

    #[test]
    fn test_flux_file_header_serialization() {
        let header = FluxFileHeader {
            n_zone: 10,
            n_fluxes: 100,
        };

        let mut buffer = Vec::new();
        header.to_bytes(&mut buffer);
        let mut offset = 0;
        let deserialized = FluxFileHeader::from_bytes(&buffer, &mut offset).unwrap();

        assert_eq!(header.n_zone, deserialized.n_zone);
        assert_eq!(header.n_fluxes, deserialized.n_fluxes);
    }

    #[test]
    fn test_scalar_file_header_serialization() {
        let header = ScalarFileHeader { n_zone: 5 };

        let mut buffer = Vec::new();
        header.to_bytes(&mut buffer);
        let mut offset = 0;
        let deserialized = ScalarFileHeader::from_bytes(&buffer, &mut offset).unwrap();

        assert_eq!(header.n_zone, deserialized.n_zone);
    }

    #[test]
    fn test_raw_flux_serialization() {
        let raw_flux = RawFlux {
            id_source: 1,
            id_target: 2,
            flux_source_target: 0.1,
            flux_target_source: 2.71,
        };

        let mut buffer = Vec::new();
        raw_flux.to_bytes(&mut buffer);
        let mut offset = 0;
        let deserialized = RawFlux::from_bytes(&buffer, &mut offset).unwrap();

        assert_eq!(raw_flux.id_source, deserialized.id_source);
        assert_eq!(raw_flux.id_target, deserialized.id_target);
        assert_eq!(raw_flux.flux_source_target, deserialized.flux_source_target);
        assert_eq!(raw_flux.flux_target_source, deserialized.flux_target_source);
    }

    #[test]
    fn test_raw_scalar_serialization() {
        let raw_scalar = RawScalar { value: 0.1 };

        let mut buffer = Vec::new();
        raw_scalar.to_bytes(&mut buffer);
        let mut offset = 0;
        let deserialized = RawScalar::from_bytes(&buffer, &mut offset).unwrap();

        assert_eq!(raw_scalar.value, deserialized.value);
    }

    #[test]
    fn test_raw_data_scalar_serialization() {
        let raw_data_scalar = RawDataScalar {
            header: ScalarFileHeader { n_zone: 5 },
            values: vec![RawScalar { value: 0.1 }, RawScalar { value: 2.71 }],
        };

        let mut buffer = Vec::new();
        raw_data_scalar.header.to_bytes(&mut buffer);
        for value in &raw_data_scalar.values {
            value.to_bytes(&mut buffer);
        }

        let mut offset = 0;
        let header = ScalarFileHeader::from_bytes(&buffer, &mut offset).unwrap();
        let mut values = Vec::new();
        while offset < buffer.len() {
            values.push(RawScalar::from_bytes(&buffer, &mut offset).unwrap());
        }

        assert_eq!(raw_data_scalar.header.n_zone, header.n_zone);
        assert_eq!(raw_data_scalar.values.len(), values.len());
        assert_eq!(raw_data_scalar.values[0].value, values[0].value);
        assert_eq!(raw_data_scalar.values[1].value, values[1].value);
    }

    #[test]
    fn test_raw_data_flux_serialization() {
        let raw_data_flux = RawDataFlux {
            header: FluxFileHeader {
                n_zone: 10,
                n_fluxes: 100,
            },
            fluxes: vec![RawFlux {
                id_source: 1,
                id_target: 2,
                flux_source_target: 0.1,
                flux_target_source: 2.71,
            }],
        };

        let mut buffer = Vec::new();
        raw_data_flux.header.to_bytes(&mut buffer);
        for flux in &raw_data_flux.fluxes {
            flux.to_bytes(&mut buffer);
        }

        let mut offset = 0;
        let header = FluxFileHeader::from_bytes(&buffer, &mut offset).unwrap();
        let mut fluxes = Vec::new();
        while offset < buffer.len() {
            fluxes.push(RawFlux::from_bytes(&buffer, &mut offset).unwrap());
        }

        assert_eq!(raw_data_flux.header.n_zone, header.n_zone);
        assert_eq!(raw_data_flux.header.n_fluxes, header.n_fluxes);
        assert_eq!(raw_data_flux.fluxes.len(), fluxes.len());
        assert_eq!(raw_data_flux.fluxes[0].id_source, fluxes[0].id_source);
        assert_eq!(raw_data_flux.fluxes[0].id_target, fluxes[0].id_target);
        assert_eq!(
            raw_data_flux.fluxes[0].flux_source_target,
            fluxes[0].flux_source_target
        );
        assert_eq!(
            raw_data_flux.fluxes[0].flux_target_source,
            fluxes[0].flux_target_source
        );
    }

    #[test]
    fn test_raw_data_trait() {
        let raw_data_scalar = RawDataScalar {
            header: ScalarFileHeader { n_zone: 5 },
            values: vec![RawScalar { value: 0.1 }, RawScalar { value: 2.71 }],
        };

        let path = "./test.raw";
        raw_data_scalar.write_raw(path).unwrap();
        let deserialized = RawDataScalar::read_raw(path).unwrap();

        assert_eq!(raw_data_scalar.header.n_zone, deserialized.header.n_zone);
        assert_eq!(raw_data_scalar.values.len(), deserialized.values.len());
        assert_eq!(
            raw_data_scalar.values[0].value,
            deserialized.values[0].value
        );
        assert_eq!(
            raw_data_scalar.values[1].value,
            deserialized.values[1].value
        );

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_raw_data_flux_write() {
        let raw_data_flux = RawDataFlux {
            header: FluxFileHeader {
                n_zone: 10,
                n_fluxes: 100,
            },
            fluxes: vec![RawFlux {
                id_source: 1,
                id_target: 2,
                flux_source_target: 0.1,
                flux_target_source: 2.71,
            }],
        };

        let path = "./tes2t.raw";
        raw_data_flux.write_raw(path).unwrap();
        let deserialized = RawDataFlux::read_raw(path).unwrap();

        assert_eq!(raw_data_flux.header.n_zone, deserialized.header.n_zone);
        assert_eq!(raw_data_flux.header.n_fluxes, deserialized.header.n_fluxes);
        assert_eq!(raw_data_flux.fluxes.len(), deserialized.fluxes.len());
        assert_eq!(
            raw_data_flux.fluxes[0].id_source,
            deserialized.fluxes[0].id_source
        );
        assert_eq!(
            raw_data_flux.fluxes[0].id_target,
            deserialized.fluxes[0].id_target
        );
        assert_eq!(
            raw_data_flux.fluxes[0].flux_source_target,
            deserialized.fluxes[0].flux_source_target
        );
        assert_eq!(
            raw_data_flux.fluxes[0].flux_target_source,
            deserialized.fluxes[0].flux_target_source
        );

        std::fs::remove_file(path).unwrap();
    }
}
