// SPDX-License-Identifier: GPL-3.0-or-later

use crate::CMError;
use cmtool_data::{
    CMCase, CMCaseJson, CMCaseWriter, CMExportType, DEFAULT_CASE_FILE_NAME, PhaseCM, RawPhase,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct GenerateContract {
    case: CMCase,
    liquid_phase: Option<RawPhase>,
    gas_phase: Option<RawPhase>,
    relative_path: Option<String>,
}

impl fmt::Debug for GenerateContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerateContract").finish()
    }
}

fn format_path_case(relative_path: &Option<String>, file_name: &str) -> PathBuf {
    relative_path
        .as_ref()
        .map(|rel| std::path::Path::new(&rel).join(file_name))
        .unwrap_or_else(|| std::path::Path::new(file_name).to_path_buf())
}

impl GenerateContract {
    pub fn get_case(&self) -> &CMCase {
        &self.case
    }

    pub fn new_single_phase(case: CMCase, phase: RawPhase, relative_path: Option<String>) -> Self {
        if phase.identifier == PhaseCM::Liquid {
            Self {
                case,
                liquid_phase: Some(phase),
                gas_phase: None,
                relative_path,
            }
        } else {
            Self {
                case,
                liquid_phase: None,
                gas_phase: Some(phase),
                relative_path,
            }
        }
    }

    pub fn new(
        case: CMCase,
        liquid_phase: RawPhase,
        gas_phase: Option<RawPhase>,
        relative_path: Option<String>,
    ) -> Self {
        Self {
            case,
            liquid_phase: Some(liquid_phase),
            gas_phase,
            relative_path,
        }
    }

    fn write_phase(
        dest: impl AsRef<std::path::Path>,
        case: &mut CMCase,
        phase: RawPhase,
        relative_path: Option<String>,
    ) -> Result<(), CMError> {
        let (flowp, volumep) = phase.write(dest.as_ref().to_str().expect("utf path"))?;

        let flowp = format_path_case(&relative_path, &flowp);
        let volumep = format_path_case(&relative_path, &volumep);

        case.add(
            CMExportType::Flow(phase.identifier).into(),
            flowp.to_str().expect("UTF-8 path"),
        );
        case.add(
            CMExportType::Volume(phase.identifier).into(),
            volumep.to_str().expect("UTF-8 path"),
        );

        Ok(())
    }

    fn prepare_fs(&self, root_dir: impl AsRef<std::path::Path>) -> Result<PathBuf, CMError> {
        let path = if let Some(p) = &self.relative_path {
            root_dir.as_ref().join(p)
        } else {
            root_dir.as_ref().to_owned()
        };
        std::fs::create_dir_all(&path)?; //FIXME
        Ok(path)
    }

    pub fn write(mut self, root_dir: impl AsRef<std::path::Path>) -> Result<CMCase, CMError> {
        if self.liquid_phase.is_none() && self.gas_phase.is_none() {
            return Err(CMError::Custom("No phase to write".to_owned()));
        }

        let path = self.prepare_fs(&root_dir)?;

        if let Some(liquid_phase) = self.liquid_phase {
            Self::write_phase(
                &path,
                &mut self.case,
                liquid_phase,
                self.relative_path.clone(),
            )?;
        }

        if let Some(gas_phase) = self.gas_phase {
            Self::write_phase(path, &mut self.case, gas_phase, self.relative_path.clone())?;
        }

        let case_path = root_dir.as_ref().join(DEFAULT_CASE_FILE_NAME);
        CMCaseJson::write_case(self.case.clone(), &case_path)?;

        Ok(self.case)
    }
}
