use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone, Copy)]
pub enum CMAExportType {
    LiquidFlow,
    GasFlow,
    GasVolume,
    LiquidVolume,
    EnergyDissipation,
    Kla,
    Other,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone, Copy)]
pub enum PhaseCM {
    Liquid,
    Gas,
}

impl PhaseCM {
    pub fn identifier(self) -> String {
        match self {
            Self::Liquid => String::from("L"),
            Self::Gas => String::from("G"),
        }
    }
}

pub enum CMExportType {
    Flow(PhaseCM),
    Volume(PhaseCM),
    EnergyDissipation,
    Kla,
    Other(String),
}

impl From<CMAExportType> for CMExportType {
    fn from(value: CMAExportType) -> Self {
        match value {
            CMAExportType::LiquidFlow => Self::Flow(PhaseCM::Liquid),
            CMAExportType::GasFlow => Self::Flow(PhaseCM::Gas),
            CMAExportType::GasVolume => Self::Volume(PhaseCM::Gas),
            CMAExportType::LiquidVolume => Self::Volume(PhaseCM::Liquid),
            CMAExportType::EnergyDissipation => Self::EnergyDissipation,
            CMAExportType::Kla => Self::Kla,
            CMAExportType::Other => Self::Other("Unnamed".to_string()),
        }
    }
}

impl From<CMExportType> for CMAExportType {
    fn from(val: CMExportType) -> Self {
        match val
        {
            // CMExportType::Flow(phase_cm) => {},
            CMExportType::Flow(PhaseCM::Gas) => CMAExportType::GasFlow,
            CMExportType::Flow(PhaseCM::Liquid) => CMAExportType::LiquidFlow,
            CMExportType::Volume(PhaseCM::Gas) => CMAExportType::GasVolume,
            CMExportType::Volume(PhaseCM::Liquid) => CMAExportType::LiquidVolume,
            CMExportType::EnergyDissipation => CMAExportType::EnergyDissipation,
            CMExportType::Kla => CMAExportType::Kla,
            CMExportType::Other(_) => CMAExportType::Other,
        }
    }
}

impl CMExportType {
    pub fn default_filename(self) -> String {
        match self {
            Self::Flow(g) => {
                format!("flow{}.raw", g.identifier())
            }
            Self::Volume(g) => {
                format!("vol{}.raw", g.identifier())
            }
            Self::EnergyDissipation => "epsturb.raw".to_string(),
            Self::Kla => "kla.raw".to_string(),
            Self::Other(name) => {
                format!("m_{}.raw", name)
            }
        }
    }
}

impl From<i8> for CMExportType {
    fn from(value: i8) -> Self {
        match value {
            0 => CMExportType::Flow(PhaseCM::Liquid),
            1 => CMExportType::Flow(PhaseCM::Gas),
            2 => CMExportType::Volume(PhaseCM::Gas),
            3 => CMExportType::Volume(PhaseCM::Liquid),
            4 => CMExportType::EnergyDissipation,
            5 => CMExportType::Kla,
            6 => CMExportType::Other(String::from("Unnamed")),
            _ => panic!("Invalid value for CMAExportType: {}", value),
        }
    }
}

impl From<i8> for CMAExportType {
    fn from(value: i8) -> Self {
        match value {
            0 => CMAExportType::LiquidFlow,
            1 => CMAExportType::GasFlow,
            2 => CMAExportType::GasVolume,
            3 => CMAExportType::LiquidVolume,
            4 => CMAExportType::EnergyDissipation,
            5 => CMAExportType::Kla,
            6 => CMAExportType::Other,
            _ => panic!("Invalid value for CMAExportType: {}", value),
        }
    }
}
