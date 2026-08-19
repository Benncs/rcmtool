// SPDX-License-Identifier: GPL-3.0-or-later



use cmtool_core::{CMHandle, CoreError, grid::MeshType, model::BalanceSettings};
use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;

pub struct CoreErrorWrapper(CoreError);

impl From<CoreError> for CoreErrorWrapper {
    fn from(value: CoreError) -> Self {
        Self(value)
    }
}

impl From<CoreErrorWrapper> for PyErr {
    fn from(error: CoreErrorWrapper) -> Self {
        let message = error.0.to_string();
        match error.0 {
            CoreError::IO(_) => PyIOError::new_err(message),
            //A divergence or an empty flow map is bad input, not an interpreter fault
            CoreError::Model(_) => PyValueError::new_err(message),
            //Keep the mapping the `data` module already uses
            CoreError::Data(data) => crate::PythonError::from(data).into(),
            CoreError::Custom(_) | CoreError::Handle => PyRuntimeError::new_err(message),
        }
    }
}

/// Compartment model over one CFD geometry.

#[pyclass(name = "CMHandle")]
pub struct CMHandleWrapper {
    handle: CMHandle,
    case: cmtool_core::ensight_gold::case::Case,
}

#[pymethods]
impl CMHandleWrapper {
    /// Reads an EnSight Gold `.encas` case and builds the model over the geometry it names
    #[new]
    fn new(py: Python<'_>, n_div: [usize; 3], case_path: &str) -> PyResult<Self> {
        py.detach(|| {
            let case = cmtool_core::ensight_gold::case::Case::read(case_path)?;
            let handle = CMHandle::init(
                n_div,
                &case.root,
                &case.geometry_file_path,
                MeshType::Cylindrical,
            )?;
            Ok::<_, CoreError>(Self { handle, case })
        })
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    /// The file one variable of the case is stored in, by the name the `.encas` declares
    fn variable(&self, name: &str) -> PyResult<String> {
        self.case
            .paths
            .iter()
            .find(|variable| variable.name == name)
            .map(|variable| {
                std::path::Path::new(&self.case.root)
                    .join(&variable.filepath)
                    .to_string_lossy()
                    .into_owned()
            })
            .ok_or_else(|| {
                PyValueError::new_err(format!("No variable named {:?} in the case", name))
            })
    }

    /// Dumps every variable the case declares
    fn dump_all(&self, py: Python<'_>, root_export: &str) -> PyResult<()> {
        py.detach(|| {
            self.handle
                .dump_all(root_export, &self.case.root, &self.case.paths)
        })
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    fn dump_scalar(&self, py: Python<'_>, res_name: &str, path: &str) -> PyResult<()> {
        py.detach(|| self.handle.dump_scalar(res_name, path))
            .map(|_| ())
            .map_err(CoreErrorWrapper::from)
            .map_err(Into::into)
    }

    /// The liquid carries what the gas leaves, `1 - gas_fraction`
    fn dump_scalar_liquid(
        &self,
        py: Python<'_>,
        res_name: &str,
        path: &str,
        gas_fraction: &str,
    ) -> PyResult<()> {
        py.detach(|| {
            self.handle.dump_scalar_fraction(
                res_name,
                path,
                self.handle.get_scalar(gas_fraction)?.scalar_shift(1.),
            )
        })
        .map(|_| ())
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    fn dump_scalar_gas(
        &self,
        py: Python<'_>,
        res_name: &str,
        path: &str,
        gas_fraction: &str,
    ) -> PyResult<()> {
        py.detach(|| {
            self.handle
                .dump_scalar_fraction(res_name, path, self.handle.get_scalar(gas_fraction)?)
        })
        .map(|_| ())
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    fn dump_vector(&self, py: Python<'_>, res_name: &str, path: &str) -> PyResult<()> {
        py.detach(|| self.handle.dump_vector(res_name, path))
            .map_err(CoreErrorWrapper::from)
            .map_err(Into::into)
    }

    /// The liquid carries what the gas leaves, `1 - gas_fraction`
    fn dump_vector_liquid(
        &self,
        py: Python<'_>,
        res_name: &str,
        path: &str,
        gas_fraction: &str,
    ) -> PyResult<()> {
        py.detach(|| {
            self.handle.dump_vector_phase_fraction(
                res_name,
                path,
                self.handle.get_scalar(gas_fraction)?.scalar_shift(1.),
            )
        })
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    fn dump_vector_gas(
        &self,
        py: Python<'_>,
        res_name: &str,
        path: &str,
        gas_fraction: &str,
    ) -> PyResult<()> {
        py.detach(|| {
            self.handle.dump_vector_phase_fraction(
                res_name,
                path,
                self.handle.get_scalar(gas_fraction)?,
            )
        })
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    fn dump_vector_from_scalar(
        &self,
        py: Python<'_>,
        res_name: &str,
        path_i: &str,
        path_j: &str,
        path_k: &str,
    ) -> PyResult<()> {
        py.detach(|| {
            self.handle
                .dump_vector_from_scalar(res_name, path_i, path_j, path_k, None)
        })
        .map(|_| ())
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    /// The liquid carries what the gas leaves, `1 - gas_fraction`
    fn dump_vector_from_scalar_liquid(
        &self,
        py: Python<'_>,
        res_name: &str,
        path_i: &str,
        path_j: &str,
        path_k: &str,
        gas_fraction: &str,
    ) -> PyResult<()> {
        py.detach(|| {
            self.handle.dump_vector_from_scalar(
                res_name,
                path_i,
                path_j,
                path_k,
                Some(self.handle.get_scalar(gas_fraction)?.scalar_shift(1.)),
            )
        })
        .map(|_| ())
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    fn dump_vector_from_scalar_gas(
        &self,
        py: Python<'_>,
        res_name: &str,
        path_i: &str,
        path_j: &str,
        path_k: &str,
        gas_fraction: &str,
    ) -> PyResult<()> {
        py.detach(|| {
            self.handle.dump_vector_from_scalar(
                res_name,
                path_i,
                path_j,
                path_k,
                Some(self.handle.get_scalar(gas_fraction)?),
            )
        })
        .map(|_| ())
        .map_err(CoreErrorWrapper::from)
        .map_err(Into::into)
    }

    fn dump_real_volume(&self, py: Python<'_>, res_name: &str) -> PyResult<()> {
        py.detach(|| self.handle.dump_real_volume(res_name))
            .map_err(CoreErrorWrapper::from)
            .map_err(Into::into)
    }

    /// Writes the compartment mesh with the given scalars on it, plus `real_volume`
    #[cfg(feature = "use_vtk")]
    fn export_vtk(
        &self,
        py: Python<'_>,
        path: &str,
        sc: Vec<(pyo3::PyRef<'_, crate::rd::RawDataScalarWrapper>, String)>,
    ) -> PyResult<()> {
        let sc: Vec<(cmtool_data::RawDataScalar, String)> =
            sc.iter().map(|(r, n)| (r.0.clone(), n.clone())).collect();

        py.detach(|| self.handle.export_vtk(path, sc))
            .map_err(CoreErrorWrapper::from)
            .map_err(Into::into)
    }

    fn dump_volume(&self, py: Python<'_>) {
        py.detach(|| self.handle.dump_volume())
    }

    fn export_geometry_compartments(&self, py: Python<'_>) {
        py.detach(|| self.handle.export_geometry_compartments())
    }

    fn real_volume(&self, py: Python<'_>) -> Py<numpy::PyArray1<f64>> {
        let volumes = py.detach(|| self.handle.real_volume());
        numpy::PyArray1::from_owned_array(py, numpy::ndarray::Array1::from(volumes)).unbind()
    }

    fn set_balance_settings(&mut self, max_iterations: usize, tolerance: f64, max_divergence: f64) {
        self.handle.set_balance_settings(BalanceSettings {
            max_iterations,
            tolerance,
            max_divergence,
        })
    }

    fn balance_settings(&self) -> (usize, f64, f64) {
        let settings = self.handle.balance_settings();
        (
            settings.max_iterations,
            settings.tolerance,
            settings.max_divergence,
        )
    }
}
