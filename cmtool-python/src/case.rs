// SPDX-License-Identifier: GPL-3.0-or-later

use cmtool_data::CMCaseWriter;
use pyo3::prelude::*;

use crate::PythonError;

//Use C compartible enum because Enum-Struct not supported by PyO3
// #[pyclass(name = "CMExportType")]
#[pyclass(from_py_object, name = "CMExportType")]
#[derive(Clone, Copy)]
pub enum CMExportTypeWrapper {
    LiquidFlow,
    GasFlow,
    GasVolume,
    LiquidVolume,
    EnergyDissipation,
    Kla,
    Other,
}

impl From<CMExportTypeWrapper> for cmtool_data::CMAExportType {
    fn from(value: CMExportTypeWrapper) -> Self {
        match value {
            CMExportTypeWrapper::LiquidFlow => Self::LiquidFlow,
            CMExportTypeWrapper::GasFlow => Self::GasFlow,
            CMExportTypeWrapper::LiquidVolume => Self::LiquidVolume,
            CMExportTypeWrapper::GasVolume => Self::GasVolume,
            CMExportTypeWrapper::EnergyDissipation => Self::EnergyDissipation,
            CMExportTypeWrapper::Kla => Self::Kla,
            CMExportTypeWrapper::Other => Self::Other,
        }
    }
}

// #[pyclass(name = "CMCase")]
#[pyclass(from_py_object, name = "CMCase")]
#[derive(Clone)]
pub struct CMCaseWrapper(cmtool_data::CMCase);

#[pymethods]
impl CMCaseWrapper {
    pub fn add(&mut self, stype: CMExportTypeWrapper, relative_path: &str) {
        let c: cmtool_data::CMAExportType = stype.into();
        self.0.add(c, relative_path);
    }

    pub fn write(&self, _py: Python<'_>, path: &str) -> PyResult<()> {
        let p = std::path::Path::new(path);
        let c = self.0.clone();
        cmtool_data::CMCaseJson::write_case(c, p).map_err(PythonError::from)?;
        Ok(())
    }
    pub fn resolve(&self, root: &str, stype: CMExportTypeWrapper) -> Option<Vec<String>> {
        self.0.resolve_all(root, stype.into())
    }
}

#[pyfunction]
pub fn make_cm_case(
    _py: Python<'_>,
    n_div: [u32; 3],
    time_per_flow_map: f64,
    description: Option<String>,
    recursive: bool,
) -> CMCaseWrapper {
    CMCaseWrapper(cmtool_data::CMCase::new(
        n_div,
        time_per_flow_map,
        description,
        recursive,
    ))
}

fn read_cm_case_gen(path: &str) -> PyResult<CMCaseWrapper> {
    let p = std::path::Path::new(path);
    let ejson = cmtool_data::read_case(p);
    let c = ejson.map_err(PythonError::from)?;
    Ok(CMCaseWrapper(c))
}

#[pyfunction]
pub fn read_cm_case(_py: Python<'_>, path: &str) -> PyResult<CMCaseWrapper> {
    read_cm_case_gen(path)
}
