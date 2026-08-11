// SPDX-License-Identifier: GPL-3.0-or-later

use ndarray::Array2;

use crate::{DataError, RawData, RawFlux, rawdata};

pub struct FlowMapDescriptor {
    pub flowmap: Array2<f64>,
    pub neighbors: Array2<usize>, //TODO
    pub volumes: Vec<f64>,
}

impl FlowMapDescriptor {
    pub fn from_path(
        data_flows_path: impl AsRef<std::path::Path>,
        data_volumes_path: impl AsRef<std::path::Path>,
    ) -> Result<Self, DataError> {
        let df = rawdata::RawDataFlux::read_raw(data_flows_path).ok_or(DataError::BadData)?;
        let dv = rawdata::RawDataScalar::read_raw(data_volumes_path).ok_or(DataError::BadData)?;

        Self::from_raw_data(&df, &dv)
    }

    pub fn from_raw_data(
        data_flows: &rawdata::RawDataFlux,
        data_volumes: &rawdata::RawDataScalar,
    ) -> Result<Self, DataError> {
        let n_zone = data_flows.header.n_zone as usize;
        let mut flowmap = Array2::<f64>::zeros((n_zone, n_zone));

        let mut neighbors: Vec<Vec<usize>> = vec![Vec::with_capacity(10); n_zone];

        if data_flows.header.n_zone != data_volumes.header.n_zone {
            return Err(DataError::BadData);
        }

        let mut add_at = |i: usize, j: usize, val: f64| -> Result<(), DataError> {
            //Error should never be triggered, by construction i<n and j<n
            let g = flowmap.get_mut((i, j)).ok_or(DataError::BadData)?;
            *g += val;
            Ok(())
        };

        for &RawFlux {
            id_source,
            id_target,
            flux_source_target,
            flux_target_source,
        } in data_flows.fluxes.iter()
        {
            let id_source = id_source as usize;
            let id_target = id_target as usize;
            assert!(flux_source_target >= 0.);
            assert!(flux_target_source >= 0.);

            add_at(id_source, id_target, flux_source_target)?;
            add_at(id_target, id_source, flux_target_source)?;

            neighbors[id_source].push(id_target);
            neighbors[id_target].push(id_source);
        }

        let max_size = neighbors
            .iter()
            .map(|val| val.len())
            .max()
            .ok_or(DataError::BadData)?;

        let mut neighbor_flat = Array2::<usize>::zeros((n_zone, max_size));

        neighbor_flat.fill(n_zone + 1); //Any ghost neighbor will have value n+1

        for (i_zone, neighbors_for_zone) in neighbors.iter().enumerate() {
            for (i_n, id_neighbor) in neighbors_for_zone.iter().enumerate() {
                // *(neighbor_flat.get_mut((i_zone, i_n)).unwrap()) = *id_neighbor;
                *(neighbor_flat
                    .get_mut((i_zone, i_n))
                    .expect("Flat neighbor out of bound")) = *id_neighbor;
            }
        }

        let volumes: Vec<f64> = data_volumes.values.iter().map(|v| v.value).collect();

        Ok(FlowMapDescriptor {
            flowmap,
            neighbors: neighbor_flat,
            volumes,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_descriptor() {
        let _flow_cma = std::env::var("CUVE_SLDMSH_FLOW_PATH");

        let _volume_cma = std::env::var("CUVE_SLDMSH_VOLUME_PATH");

        if let (Ok(flow_cma), Ok(volume_cma)) = (_flow_cma, _volume_cma) {
            let descriptor = FlowMapDescriptor::from_path(flow_cma, volume_cma).unwrap();

            assert!(!descriptor.volumes.is_empty());
            assert!(descriptor.flowmap.is_square());
            assert!(descriptor.volumes.len() == descriptor.flowmap.ncols());
            assert!(descriptor.neighbors.nrows() == descriptor.flowmap.ncols());
        }
    }
}
