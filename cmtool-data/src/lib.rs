// SPDX-License-Identifier: GPL-3.0-or-later

mod case;
mod descriptors;
mod flowmap;
mod rawdata;
mod states;
mod transitionner;
pub use case::{CCMCaseInfo, CMCase, CMCaseJson, CMCaseReader, CMCaseWriter};
pub use descriptors::{CMAExportType, CMExportType, PhaseCM};
pub use flowmap::FlowMapDescriptor;
pub use rawdata::{
    FluxFileHeader, RawData, RawDataFlux, RawDataScalar, RawFlux, RawPhase, RawScalar,
    ScalarFileHeader, ScalarValueType,
};
pub use states::*;
use std::io;
use thiserror::Error;
pub use transitionner::*;
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

    #[error("Data is illed-format")]
    BadData,
}

#[inline(always)]
fn linear_index_row_major(_n_row: usize, n_col: usize, i: usize, j: usize) -> usize {
    i * n_col + j
}

#[inline(always)]
fn linear_index_col_major(n_row: usize, _n_col: usize, i: usize, j: usize) -> usize {
    j * n_row + i
}
