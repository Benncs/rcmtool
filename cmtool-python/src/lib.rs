use cmtool_data::RawData;
use pyo3::prelude::*;

#[pyclass(name = "RawDataScalar")]
pub struct RawDataScalarWrapper(cmtool_data::RawDataScalar);
#[pymethods]
impl RawDataScalarWrapper {
    #[getter]
    fn n_zone(&self) -> usize {
        self.0.header.n_zone as usize
    }
}

#[pyclass(name = "RawDataFlux")]
pub struct RawDataFluxWrapper(cmtool_data::RawDataFlux);
#[pymethods]
impl RawDataFluxWrapper {
    #[getter]
    fn n_zone(&self) -> usize {
        self.0.header.n_zone as usize
    }
}

#[pyfunction]
fn read_rawflow(path: &str) -> RawDataFluxWrapper {
    RawDataFluxWrapper(cmtool_data::RawDataFlux::read_raw(path).unwrap())
}

#[pyfunction]
fn read_rawscalar(path: &str) -> RawDataScalarWrapper {
    RawDataScalarWrapper(cmtool_data::RawDataScalar::read_raw(path).unwrap())
}

#[pymodule]
mod pycmtool {
    #[pymodule_export]
    use super::{read_rawflow,read_rawscalar};
}
