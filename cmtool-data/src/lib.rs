// SPDX-License-Identifier: GPL-3.0-or-later

mod case;
mod descriptors;
mod flowmap;
mod rawdata;
mod states;
mod transitioner;
pub use case::{CCMCaseInfo, CMCase, CMCaseJson, CMCaseReader, CMCaseWriter, read_case};
use core::f64;
pub use descriptors::{CMAExportType, CMExportType, PhaseCM};
pub use flowmap::FlowMapDescriptor;
pub use rawdata::{
    FluxFileHeader, RawData, RawDataFlux, RawDataScalar, RawFlux, RawPhase, RawScalar,
    ScalarFileHeader, ScalarValueType,
};
pub use states::*;
use std::io;
use thiserror::Error;
pub use transitioner::*;

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
#[allow(unused)]
fn linear_index_row_major(_n_row: usize, n_col: usize, i: usize, j: usize) -> usize {
    i * n_col + j
}

#[inline(always)]
#[allow(unused)]
fn linear_index_col_major(n_row: usize, _n_col: usize, i: usize, j: usize) -> usize {
    j * n_row + i
}

///Create transitioner
pub fn get_transitioner<T: FlowMapTransitioner>(root: &str) -> Result<T, DataError> {
    let case_path = format!("{}/cma_case", root);
    let p = std::path::Path::new(&case_path);
    let case = read_case(p)?;

    //Load all the case information into the iterator
    T::from_case(root, &case)
}

///Compute the smallest average residence time in compartment
pub fn get_min_residence_time<T: FlowMapTransitioner>(fmt: &T) -> f64 {
    let n_states = fmt.size();
    let mut min_all = f64::MAX;

    for i_state in 0..n_states {
        let state = fmt
            .get_at(i_state)
            .expect("Transitioner error: n_state != real stored states");

        if state.liquid.out_flows.len() != state.liquid.volumes.len() {
            panic!("Mismatched lengths between out_flows and volumes.");
        }

        let min_i = state
            .liquid
            .out_flows
            .iter()
            .zip(state.liquid.volumes.iter())
            .map(|(&f, &v)| f / v)
            .filter(|&x| x.is_finite())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater))
            .expect("Should exist a minimum for the state");

        min_all = f64::min(min_all, min_i);
    }

    min_all
}
