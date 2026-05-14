// SPDX-License-Identifier: GPL-3.0-or-later

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CMError {
    #[error("Cmtool encountered an unknown error. Please check the input and try again.")]
    Default,

    #[error("Cmtool error: {0}")]
    Custom(String),

    #[error(
        "Cmtool: Mass balance error in PFR '{0}' with phase {1}. Check your inputs or calculations."
    )]
    MassBalance(String, String),

    #[error("Cmtool data error: {0}. Ensure your data files are correct and accessible.")]
    Data(#[from] cmtool_data::DataError),

    #[error("I/O error: Failed to handle the file '{0}'. Check file path and permissions.")]
    IO(#[from] io::Error),

    #[error(
        "Cmtool parse error: {0}. Verify that the XML input is well-formed and matches expected schema."
    )]
    Parse(#[from] serde_xml_rs::Error),
}
