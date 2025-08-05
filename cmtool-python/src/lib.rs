use cmtool_data::RawData;
use numpy::ndarray::{self, Array2};
use numpy::{IntoPyArray, PyArray1};
use numpy::{PyArray2, PyArrayMethods};
use pyo3::prelude::*;

#[pyclass(name = "FlowMapDescriptor")]
pub struct FlowMapDescriptorWrapper(cmtool_data::FlowMapDescriptor);

#[pyclass(name = "RawDataScalar")]
pub struct RawDataScalarWrapper(cmtool_data::RawDataScalar);
#[pymethods]
impl RawDataScalarWrapper {
    #[getter]
    fn n_zone(&self) -> usize {
        self.0.header.n_zone as usize
    }

    #[getter]
    fn data(&self, py: Python<'_>) -> Py<PyArray1<f64>> {
        let values: Vec<f64> = self.0.values.iter().map(|f| f.value).collect();

        let array = ndarray::Array1::from(values);

        PyArray1::from_owned_array(py, array).unbind()
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
fn read_flowmap(py: Python<'_>, path: &str) -> FlowMapDescriptorWrapper {
    let f = cmtool_data::RawDataFlux::read_raw(path).unwrap();
    let fm = cmtool_data::FlowMapDescriptor::from_raw_data(&f).unwrap();
    FlowMapDescriptorWrapper(fm)
}

#[pymethods]
impl FlowMapDescriptorWrapper {
    //TODO check safety of this, maybe use RC<refcell> to do not have rust mutability
    #[getter]
    fn flowmap<'py>(this: Bound<'py, Self>) -> Bound<'py, PyArray2<f64>> {
        let flowmap = &this.borrow().0.flowmap;

        // SAFETY:
        // - The returned NumPy array shares memory with the internal `flowmap` (Array2<f64>).
        // - We use `borrow_from_array`, which ties the array's lifetime to the Python object (`this`).
        // - This guarantees that the underlying Rust memory remains valid as long as Python holds the array.
        //
        // Critical Requirements:
        // - `self.0.flowmap` must not be mutated in a way that causes memory reallocation (e.g., replacing it).
        //   For example, the following code is unsafe if it runs after `pyobject.flowmap` is accessed:
        //
        //     fn drop(&mut self) {
        //         self.0.flowmap = Array2::zeros((1, 1));  // BAD: reallocates backing buffer
        //     }
        //
        // - Violating this invariant (e.g., replacing the array or shrinking it) while Python holds a reference
        //   will cause undefined behavior (likely a segmentation fault).
        //
        // - Only expose immutable views or ensure exclusive access if mutations are needed.
        unsafe { PyArray2::borrow_from_array(flowmap, this.into_any()) }
    }
}

#[pymodule]
mod pycmtool {
    #[pymodule_export]
    use super::{read_flowmap, read_rawflow, read_rawscalar};
}
