mod case;
mod descriptors;
mod rawdata;
pub use case::{CCMCaseInfo, CMCase, CMCaseJson, CMCaseReader, CMCaseWriter};
pub use descriptors::{CMAExportType, CMExportType, PhaseCM};
pub use rawdata::{
    FluxFileHeader, RawData, RawDataFlux, RawDataScalar, RawFlux, RawPhase, RawScalar,
    ScalarFileHeader, ScalarValueType,
};
use std::io;

use ndarray::Array2;
use thiserror::Error;

/// Errors that can occur during data operations.
///
/// This enum encapsulates various error conditions that might arise during
/// data reading, writing, and serialization operations.
#[derive(Error, Debug)]
pub enum DataError {
    /// I/O error variant that occurs during file reading or writing operations.
    ///
    /// # Arguments
    ///
    /// * `source` - The underlying IO error that triggered this error.
    #[error("I/O error occurred while handling the file: {0}")]
    IO(#[from] io::Error),

    /// Serialization or deserialization error variant.
    ///
    /// This error occurs when there is a failure during the serialization
    /// or deserialization of data.
    #[error("Serialization/Deserialization error occurred")]
    Serde,

    /// An unexpected or unknown error occurred during data operations.
    ///
    /// This variant serves as a catch-all for errors not explicitly covered
    /// by other variants.
    #[error("An unknown error occurred during data operation")]
    Unknown,
}

#[inline(always)]
fn linear_index_row_major(_n_row: usize, n_col: usize, i: usize, j: usize) -> usize {
    i * n_col + j
}

#[inline(always)]
fn linear_index_col_major(n_row: usize, _n_col: usize, i: usize, j: usize) -> usize {
    j * n_row + i
}


pub struct FlowMapDescriptor {
    pub flowmap: Array2<f64>,
    neighbors: Vec<Vec<usize>>, //TODO
}

impl FlowMapDescriptor {
    pub fn from_raw_data(data: &rawdata::RawDataFlux) -> Result<Self, ()> {
        let n_zone = data.header.n_zone as usize;
        let mut flowmap = Array2::<f64>::zeros((n_zone, n_zone));

        let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n_zone]; //Vec::with_capacity(data.header.n_zone as usize)

        for RawFlux {
            id_source,
            id_target,
            flux_source_target,
            flux_target_source,
        } in data.fluxes.iter()
        {
            let id_source = *id_source as usize;
            let id_target = *id_target as usize;

            if let Some(g) = flowmap.get_mut((id_source, id_target)) {
                *g += flux_source_target;
            }

            if let Some(g) = flowmap.get_mut((id_target, id_source)) {
                *g += flux_target_source;
            }
            neighbors[id_source].push(id_target);
            neighbors[id_target].push(id_source)
        }

        Ok(FlowMapDescriptor { flowmap, neighbors })
    }
}
