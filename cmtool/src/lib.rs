// SPDX-License-Identifier: GPL-3.0-or-later

mod sanitizer;
pub use sanitizer::*;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum CmtoolError {
    #[error("Cmtool: {0}")]
    Data(#[from] cmtool_data::DataError),

    #[error("Cmtool: {0}")]
    Core(#[from] cmtool_core::CoreError),

    #[error("Cmtool Assemble: {0}")]
    Assemble(#[from] cmtool_assemble::CMError),

    #[error("Cmtool: {0}")]
    Custom(String),
}