mod rawdata;

use std::{fmt::Formatter, os::{linux::raw, unix::raw::gid_t}};

pub use rawdata::{
    FluxFileHeader, RawDataFlux, RawDataScalar, RawFlux, RawScalar, ScalarFileHeader,
};

pub enum Layout {
    RowMajor,
    ColMajor,
}

pub struct LightView2D<'a> {
    non_owning_data: &'a mut [f64],
    n_row: usize,
    n_col: usize,
    layout: Layout,
}

impl<'a> LightView2D<'a> {
    pub fn new(data: &'a mut [f64], n_row: usize, n_col: usize, layout: Layout) -> Self {
        assert_eq!(
            data.len(),
            n_row * n_col,
            "Data length does not match the specified dimensions"
        );
        LightView2D {
            non_owning_data: data,
            n_row,
            n_col,
            layout,
        }
    }

    pub fn get(&self, i: usize, j: usize) -> Option<f64> {
        if i >= self.n_row || j >= self.n_col {
            return None;
        }

        let index = match self.layout {
            Layout::RowMajor => i * self.n_col + j,
            Layout::ColMajor => j * self.n_row + i,
        };

        Some(self.non_owning_data[index])
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> Option<&mut f64> {
        if i >= self.n_row || j >= self.n_col {
            return None;
        }

        let index = match self.layout {
            Layout::RowMajor => i * self.n_col + j,
            Layout::ColMajor => j * self.n_row + i,
        };

        Some(&mut self.non_owning_data[index])
    }

    pub fn set(&mut self, i: usize, j: usize, value: f64) -> Result<(), &'static str> {
        if i >= self.n_row || j >= self.n_col {
            return Err("Index out of bounds");
        }

        let index = match self.layout {
            Layout::RowMajor => i * self.n_col + j,
            Layout::ColMajor => j * self.n_row + i,
        };

        self.non_owning_data[index] = value;
        Ok(())
    }
}

struct FlowMap {
    internal_data: Vec<f64>,
    n_row: usize,
}

impl FlowMap {
    fn new(n_row: usize) -> Self {
        Self {
            internal_data: Vec::with_capacity(n_row * n_row),
            n_row,
        }
    }

    fn get_view(&mut self) -> LightView2D<'_> {
        LightView2D::new(
            &mut self.internal_data,
            self.n_row,
            self.n_row,
            Layout::RowMajor,
        )
    }
}

struct FlowMapDescriptor {
    flowmap: FlowMap,
    neighbors: Vec<Vec<usize>>, //TODO
}

impl FlowMapDescriptor {
    fn from_raw_data(data: &rawdata::RawDataFlux) -> Result<Self, ()> {
        let mut flowmap = FlowMap::new(data.header.n_zone as usize);

        let mut neighbors: Vec<Vec<usize>> = Vec::with_capacity(data.header.n_zone as usize);

        //Scope to drop the view 
        {
            let mut view = flowmap.get_view();

            for rawflux in data.fluxes.iter() {
                let id_source = rawflux.id_source as usize;
                let id_target = rawflux.id_target as usize;

                if let Some(g) = view.get_mut(id_source, id_target) {
                    *g += rawflux.flux_source_target;
                }

                if let Some(g) = view.get_mut(id_target, id_source) {
                    *g += rawflux.flux_target_source;
                }

                neighbors[id_source].push(id_target);
                neighbors[id_target].push(id_source);
                //todo
            }
        }

        

        Ok(FlowMapDescriptor { flowmap, neighbors })
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
            n_max: 100,
        };

        let mut buffer = Vec::new();
        header.to_bytes(&mut buffer);
        let mut offset = 0;
        let deserialized = FluxFileHeader::from_bytes(&buffer, &mut offset).unwrap();

        assert_eq!(header.n_zone, deserialized.n_zone);
        assert_eq!(header.n_max, deserialized.n_max);
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
                n_max: 100,
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
        assert_eq!(raw_data_flux.header.n_max, header.n_max);
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
                n_max: 100,
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
        assert_eq!(raw_data_flux.header.n_max, deserialized.header.n_max);
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
