use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

pub type ScalarValueType = f64;

#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub struct FluxFileHeader {
    pub n_zone: u32,
    pub n_fluxes: u32,
}

#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub struct ScalarFileHeader {
    pub n_zone: u32,
}

#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub struct RawFlux {
    pub id_source: u32,
    pub id_target: u32,
    pub flux_source_target: f64,
    pub flux_target_source: f64,
}

#[repr(C)]
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct RawScalar {
    pub value: f64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RawDataScalar {
    pub header: ScalarFileHeader,
    pub values: Vec<RawScalar>,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct RawDataFlux {
    pub header: FluxFileHeader,
    pub fluxes: Vec<RawFlux>,
}

impl From<f64> for RawScalar {
    fn from(value: f64) -> Self {
        Self { value }
    }
}

pub trait RawData: Sized {
    fn read_raw(path: impl AsRef<std::path::Path>) -> Option<Self>;
    fn write_raw(&self, path: &str) -> Result<(), ()>;
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

impl RawDataFlux {
    pub fn new(n_zone: usize, n_fluxes: usize) -> Self {
        Self {
            header: FluxFileHeader {
                n_zone: n_zone as u32,
                n_fluxes: n_fluxes as u32,
            },
            fluxes: vec![RawFlux::default(); n_zone],
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

    fn write_raw(&self, path: &str) -> Result<(), ()> {
        let mut file = File::create(Path::new(path)).map_err(|_| ())?;
        let mut buffer = Vec::new();

        self.header.to_bytes(&mut buffer);
        for value in &self.values {
            value.to_bytes(&mut buffer);
        }

        file.write_all(&buffer).map_err(|_| ())
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

    fn write_raw(&self, path: &str) -> Result<(), ()> {
        let mut file = File::create(Path::new(path)).map_err(|_| ())?;
        let mut buffer = Vec::new();
        self.header.to_bytes(&mut buffer);
        for flux in &self.fluxes {
            flux.to_bytes(&mut buffer);
        }

        file.write_all(&buffer).map_err(|e| {
            eprintln!("{}", e);
        })
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
