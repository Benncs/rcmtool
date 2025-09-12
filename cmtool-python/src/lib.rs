// SPDX-License-Identifier: GPL-3.0-or-later

mod case;
mod rd;
mod transitionner;

use cmtool_data::DataError;
use pyo3::prelude::*;

use pyo3::exceptions::PyRuntimeError;

#[derive(Debug)]
struct PythonError(DataError);

impl From<PythonError> for PyErr {
    fn from(error: PythonError) -> Self {
        PyRuntimeError::new_err(error.0.to_string())
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
            DiscontinuousTransitionerWrapper, IterationStateWrapper, get_transitionner,
            read_flowmap, read_rawflow, read_rawscalar,
        };
    }

    #[pymodule]
    mod case {
        #[pymodule_export]
        use super::_c::{CMCaseWrapper, c_read_cm_case, make_cm_case, read_cm_case};
    }
}
