// SPDX-License-Identifier: GPL-3.0-or-later

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Cmtool: {0}")]
    Data(#[from] cmtool_data::DataError),

    #[error("Cmtool: {0}")]
    Custom(String),

    #[error("Handle")]
    Handle,

    #[error("Error writing/reading file: {0}")]
    IO(#[from] std::io::Error),
}
