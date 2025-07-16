use cmtool_data::RawData;
use pyo3::prelude::*;
use numpy::PyArray2;

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


#[pyfunction]
fn read_flowmap( py: Python<'_>,path: &str) -> Py<PyArray2<f64>> {
    let f= cmtool_data::RawDataFlux::read_raw(path).unwrap();
    let fm = cmtool_data::FlowMapDescriptor::from_raw_data(&f).unwrap();
    PyArray2::from_owned_array(py, fm.flowmap).unbind()
}

#[pymodule]
mod pycmtool {
    #[pymodule_export]
    use super::{read_rawflow,read_rawscalar,read_flowmap};
}
