use std::sync::Arc;

use cmtool_data::{CCMCaseInfo, CMCaseReader, DiscontinuousTransitioner, FlowMapTransitioner};
use numpy::PyArray1;
use numpy::ndarray::{self};
use pyo3::prelude::*;

#[pyclass(name = "DiscontinuousTransitioner")]
pub struct DiscontinuousTransitionerWrapper(DiscontinuousTransitioner);

#[pymethods]
impl DiscontinuousTransitionerWrapper {
    fn advance(&mut self, current_time: f64, time_step: f64) -> IterationStateWrapper {
        IterationStateWrapper(self.0.advance_arc(current_time, time_step))
    }

    fn need_advance(&self, current_time: f64, time_step: f64) -> bool {
        self.0.need_advance(current_time, time_step)
    }

    fn get_current(&self) -> IterationStateWrapper {
        IterationStateWrapper(self.0.get_current_arc())
    }

    #[getter]
    fn n_flowmaps(&self) -> usize {
        self.0.size()
    }

    fn get_at(&self, idx: usize) -> IterationStateWrapper {
        if let Some(opt) = self.0.get_at(idx) {
            IterationStateWrapper(opt)
        } else {
            panic!("TODO")
        }
    }
}

#[pyclass(name = "IterationState", frozen)]
pub struct IterationStateWrapper(Arc<cmtool_data::IterationState>);

#[pyclass(name = "HydroState", frozen)]
pub struct HydroStateWrapper(*const cmtool_data::HydroState);

unsafe impl Sync for HydroStateWrapper {}
unsafe impl Send for HydroStateWrapper {}

#[pymethods]
impl HydroStateWrapper {
    #[getter]
    pub fn transition(&self, py: Python<'_>) -> (&[usize], &[usize], &[f64]) {
        let t = unsafe { (*self.0).get_transition() };
        (t.row_indices(), t.col_indices(), t.values())
    }
    #[getter]
    pub fn volumes(&self, py: Python<'_>) -> Py<PyArray1<f64>> {
        let deref = unsafe { &*self.0 };
        let array = deref.get_volume();
        let r = PyArray1::from_slice(py, array);
        r.unbind()
    }
}

#[pymethods]
impl IterationStateWrapper {
    #[getter]
    pub fn liquid(&self) -> HydroStateWrapper {
        let l = &self.0.liquid;
        HydroStateWrapper(l)
    }

    #[getter]
    fn n_compartments(&self) -> usize {
        self.0.n_compartments()
    }

    #[getter]
    fn get_gas(&self) -> Option<HydroStateWrapper> {
        self.0.gas.as_ref().map(|t| HydroStateWrapper(t))
    }

    fn misc(&self, key: &str, py: Python<'_>) -> Option<Py<PyArray1<f64>>> {
        if let Some(opt_m) = self.0.get(key) {
            let r = PyArray1::from_slice(py, opt_m);
            let a = r.unbind();
            Some(a)
        } else {
            None
        }
    }

    fn has_gas(&self) -> bool {
        self.0.gas.is_some()
    }
}

#[pyfunction]
pub fn get_transitioner(root: &str) -> DiscontinuousTransitionerWrapper {
    let t: DiscontinuousTransitioner = cmtool_data::get_transitioner(root).unwrap();
    DiscontinuousTransitionerWrapper(t)
}
