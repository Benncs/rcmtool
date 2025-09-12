// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use cmtool_data::{CMCaseReader, CMCaseWriter};
use pyo3::prelude::*;

use crate::PythonError;

//Use C compartible enum because Enum-Struct not supported by PyO3
#[pyclass(name = "CMExportType")]
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
            CMExportTypeWrapper::EnergyDissipation => Self::EnergyDissipation,
            CMExportTypeWrapper::Kla => Self::Kla,
            _ => Self::Other,
        }
    }
}

#[pyclass(name = "CMCase")]
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

fn read_cm_case_gen<T: CMCaseReader>(path: &str) -> PyResult<CMCaseWrapper> {
    let p = std::path::Path::new(path);
    let ejson = T::read_case(p);
    let c = ejson.map_err(PythonError::from)?;
    Ok(CMCaseWrapper(c))
}

#[pyfunction]
pub fn read_cm_case(_py: Python<'_>, path: &str) -> PyResult<CMCaseWrapper> {
    read_cm_case_gen::<cmtool_data::CMCaseJson>(path)
}

#[pyfunction]
#[deprecated(note = "Use Json instead")]
pub fn c_read_cm_case(_py: Python<'_>, path: &str) -> PyResult<CMCaseWrapper> {
    eprintln!("C Data format is deprecated");
    read_cm_case_gen::<cmtool_data::CCMCaseInfo>(path)
}
