// SPDX-License-Identifier: GPL-3.0-or-later

use cmtool_data::FluxFileHeader;
use cmtool_data::RawData;
use cmtool_data::RawDataFlux;
use cmtool_data::RawDataScalar;
use cmtool_data::RawFlux;
use numpy::PyArray1;
use numpy::PyArray2;
use numpy::PyUntypedArrayMethods;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
/* Scalar */

#[pyclass(name = "RawDataScalar", frozen)]
pub struct RawDataScalarWrapper(cmtool_data::RawDataScalar);
#[pymethods]
impl RawDataScalarWrapper {
    #[getter]
    pub fn n_zone(&self) -> usize {
        self.0.header.n_zone as usize
    }

    #[getter]
    pub fn data(&self, py: Python<'_>) -> Py<PyArray1<f64>> {
        let values: Vec<f64> = self.0.values.iter().map(|f| f.value).collect();
        //from does not perform copy
        let array = numpy::ndarray::Array1::from(values);
        PyArray1::from_owned_array(py, array).unbind()
    }

    pub fn write(&self, path: &str) -> PyResult<()> {
        if self.0.write_raw(path).is_ok() {
            Ok(())
        } else {
            Err(PyValueError::new_err("Scalar not found"))
        }
    }
}

#[pyfunction]
pub fn read_rawscalar(path: &str) -> PyResult<RawDataScalarWrapper> {
    if let Some(sc) = cmtool_data::RawDataScalar::read_raw(path) {
        Ok(RawDataScalarWrapper(sc))
    } else {
        Err(PyValueError::new_err("Scalar not found"))
    }
}

#[pyfunction]
pub fn scalar_from_data<'py>(
    _py: Python<'py>,
    x: numpy::PyReadonlyArrayDyn<'py, f64>,
) -> PyResult<RawDataScalarWrapper> {
    if x.shape().len() != 1 {
        return Err(PyValueError::new_err("Input array must be 1D."));
    }

    if !x.is_contiguous() {
        return Err(PyValueError::new_err(
            "Input array must be contiguous in memory.",
        ));
    }

    match x.as_slice() {
        Ok(slice) => {
            //TODO Why this condition has been used ?
            // if slice.len() != 1 {
            //     return Err(PyValueError::new_err(
            //         "Input array must contain exactly one element.",
            //     ));
            // }
            let scalar = RawDataScalar::from(slice);
            Ok(RawDataScalarWrapper(scalar))
        }
        Err(_) => Err(PyValueError::new_err(
            "Failed to convert the array to a contiguous slice.",
        )),
    }
}

/* Flows */

#[pyclass(from_py_object, name = "RawFlux")]
#[derive(Clone, Copy)]
pub struct RawFluxWrapper(cmtool_data::RawFlux);

#[pyfunction]
pub fn new_raw_flux(
    _py: Python<'_>,
    id_source: usize,
    id_target: usize,
    flux_source_target: f64,
    flux_target_source: f64,
) -> RawFluxWrapper {
    RawFluxWrapper(RawFlux {
        id_source: id_source as u32,
        id_target: id_target as u32,
        flux_source_target,
        flux_target_source,
    })
}

#[pyclass(name = "RawDataFlux")]
pub struct RawDataFluxWrapper(cmtool_data::RawDataFlux);

#[pymethods]
impl RawDataFluxWrapper {
    #[getter]
    pub fn n_zone(&self) -> usize {
        self.0.header.n_zone as usize
    }

    pub fn write(&self, path: &str) -> PyResult<()> {
        if self.0.write_raw(path).is_ok() {
            Ok(())
        } else {
            Err(PyValueError::new_err("Vector not found"))
        }
    }
}

#[pyfunction]
pub fn read_rawflow(path: &str) -> RawDataFluxWrapper {
    RawDataFluxWrapper(cmtool_data::RawDataFlux::read_raw(path).unwrap())
}

// impl FromPyObject for &[RawFluxWrapper]
// {
//   type Error=
//   fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {

//   }
// }

#[pyfunction]
pub fn vector_from_data<'py>(
    _py: Python<'py>,
    n_zone: usize,
    x: Vec<RawFluxWrapper>,
) -> PyResult<RawDataFluxWrapper> {
    let value: Vec<RawFlux> = x.iter().map(|i| i.0).collect();
    let rd = RawDataFlux {
        header: FluxFileHeader {
            n_fluxes: value.len() as u32,
            n_zone: n_zone as u32,
        },
        fluxes: value,
    };

    Ok(RawDataFluxWrapper(rd))
}

#[pyclass(name = "FlowMapDescriptor")]
pub struct FlowMapDescriptorWrapper(cmtool_data::FlowMapDescriptor);
#[pymethods]
impl FlowMapDescriptorWrapper {
    //TODO check safety of this, maybe use RC<refcell> to do not have rust mutability
    #[getter]
    pub fn flowmap(this: Bound<'_, Self>) -> Bound<'_, PyArray2<f64>> {
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

    #[getter]
    pub fn neighbors(this: Bound<'_, Self>) -> Bound<'_, PyArray2<usize>> {
        let flowmap = &this.borrow().0.neighbors;
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

    #[getter]
    pub fn volumes(&self, py: Python<'_>) -> Py<PyArray1<f64>> {
        let array = numpy::ndarray::Array1::from(self.0.volumes.clone());
        PyArray1::from_owned_array(py, array).unbind()
    }
}

#[pyfunction]
pub fn read_flowmap(_py: Python<'_>, path: &str, path_2: &str) -> FlowMapDescriptorWrapper {
    let f = cmtool_data::RawDataFlux::read_raw(path).unwrap();
    let v = cmtool_data::RawDataScalar::read_raw(path_2).unwrap();
    let fm = cmtool_data::FlowMapDescriptor::from_raw_data(&f, &v).unwrap();
    FlowMapDescriptorWrapper(fm)
}
