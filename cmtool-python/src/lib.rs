// SPDX-License-Identifier: GPL-3.0-or-later

use cmtool_data::{
    CCMCaseInfo, CMCaseReader, DiscontinuousTransitioner, FlowMapDescriptor, FlowMapTransitionner,
    RawData,
};
use numpy::PyArray1;
use numpy::ndarray::{self};
use numpy::{PyArray2, PyArrayMethods};
use pyo3::prelude::*;

mod rd;
pub use rd::*;

mod transitionner;
pub use transitionner::*;

#[pymodule]
mod pycmtool {
    #[pymodule_export]
    use super::{
        DiscontinuousTransitionerWrapper, IterationStateWrapper, get_transitionner, read_flowmap,
        read_rawflow, read_rawscalar,
    };
}
