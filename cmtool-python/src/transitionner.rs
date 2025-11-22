use std::sync::Arc;

use cmtool_data::{CCMCaseInfo, CMCaseReader, DiscontinuousTransitioner, FlowMapTransitioner};
use numpy::PyArray1;
use numpy::ndarray::{self};
use pyo3::prelude::*;

#[pyclass(name = "DiscontinuousTransitioner")]
pub struct DiscontinuousTransitionerWrapper(DiscontinuousTransitioner);

#[pyclass(name = "IterationState")]
pub struct IterationStateWrapper(Arc<cmtool_data::IterationState>);

#[pymethods]
impl IterationStateWrapper {
    #[getter]
    pub fn flowmap(&self, py: Python<'_>) -> (&[usize], &[usize], &[f64]) {
        let t = &self.0.liquid.transition;
        (t.row_indices(), t.col_indices(), t.values())
    }

    #[getter]
    pub fn volumes(&self, py: Python<'_>) -> Py<PyArray1<f64>> {
        let array = ndarray::Array1::from_vec(self.0.liquid.volumes.clone());
        PyArray1::from_owned_array(py, array).unbind()
    }
}

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

#[pyfunction]
pub fn get_transitioner(root: &str) -> DiscontinuousTransitionerWrapper {
    let t: DiscontinuousTransitioner = cmtool_data::get_transitioner(root).unwrap();
    DiscontinuousTransitionerWrapper(t)
}
