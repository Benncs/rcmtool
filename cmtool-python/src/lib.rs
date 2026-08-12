// SPDX-License-Identifier: GPL-3.0-or-later

mod case;
mod rd;
mod transitionner;

use cmtool_data::DataError;
use pyo3::prelude::*;

use pyo3::exceptions::{PyIOError, PyIndexError, PyRuntimeError, PyValueError};

#[derive(Debug)]
struct PythonError(DataError);

impl From<PythonError> for PyErr {
    fn from(error: PythonError) -> Self {
        let message = error.0.to_string();
        match error.0 {
            DataError::IO(_) => PyIOError::new_err(message),
            DataError::Serde | DataError::BadData => PyValueError::new_err(message),
            DataError::OutOfRange { .. } => PyIndexError::new_err(message),
            DataError::Unknown => PyRuntimeError::new_err(message),
        }
    }
}

impl From<DataError> for PythonError {
    fn from(other: DataError) -> Self {
        Self(other)
    }
}

#[pymodule]
mod pycmtool {
    use super::case as _c;
    use super::rd::*;
    use super::transitionner::*;
    use pyo3::pymodule;

    #[pymodule]
    mod data {

        #[pymodule_export]
        use super::{
            DiscontinuousTransitionerWrapper, IterationStateWrapper, get_transitioner,
            new_raw_flux, read_flowmap, read_rawflow, read_rawscalar, scalar_from_data,
            vector_from_data,
        };
    }

    #[pymodule]
    mod case {
        #[pymodule_export]
        use super::_c::{CMCaseWrapper, CMExportTypeWrapper, make_cm_case, read_cm_case};
    }
}
