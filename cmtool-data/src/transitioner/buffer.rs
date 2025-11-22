use crate::{CMCase, DataError, FlowMapDescriptor, IterationState, RawData, rawdata};
use std::{collections::HashMap, ops::Index, sync::Arc};

pub struct RawStateBuffer(
    FlowMapDescriptor,
    Option<FlowMapDescriptor>,
    HashMap<String, Vec<f64>>,
);

pub struct FlowMapBuffer(
    Vec<FlowMapDescriptor>,
    Option<Vec<FlowMapDescriptor>>,
    Vec<HashMap<String, Vec<f64>>>,
);

impl FlowMapBuffer {
    pub(crate) fn new_unique(RawStateBuffer(l, gas, m): RawStateBuffer) -> Self {
        Self(vec![l], gas.map(|g| vec![g]), vec![m])
    }

    pub(crate) fn new(buffers: Vec<RawStateBuffer>) -> Option<Self> {
        if buffers.is_empty() {
            return None;
        }

        let mut liq = Vec::with_capacity(buffers.len());
        let mut gas = Vec::with_capacity(buffers.len());
        let mut misc = Vec::with_capacity(buffers.len());

        let mut has_gas: Option<bool> = None;

        for RawStateBuffer(l, g, m) in buffers {
            liq.push(l);
            misc.push(m);

            match g {
                Some(gs) => {
                    match has_gas {
                        None => has_gas = Some(true),
                        Some(false) => return None, // mixed = invalid
                        _ => {}
                    }
                    gas.push(gs);
                }
                None => {
                    match has_gas {
                        None => has_gas = Some(false),
                        Some(true) => return None, // mixed = invalid
                        _ => {}
                    }
                }
            }
        }

        let gas_out = match has_gas {
            Some(true) => Some(gas),
            Some(false) => None,
            None => None,
        };

        Some(Self(liq, gas_out, misc))
    }
    pub fn into_state_buffer(self) -> Vec<Arc<IterationState>> {
        match self.1 {
            Some(gas) => self
                .0
                .into_iter()
                .zip(gas)
                .zip(self.2)
                .map(|((liq, _gas), _misc)| Arc::new(IterationState::new(liq, Some(_gas), _misc)))
                .collect(),
            None => self
                .0
                .into_iter()
                .zip(self.2)
                .map(|(fd, misc)| Arc::new(IterationState::new(fd, None, misc)))
                .collect(),
        }
    }
}

impl Index<usize> for FlowMapBuffer {
    type Output = FlowMapDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub(super) fn read_descriptors(root: &str, case: &CMCase) -> Result<RawStateBuffer, DataError> {
    let mut misc = HashMap::new();

    //TODO Improve
    if let Some(eps_path) = case.resolve(root, crate::CMAExportType::EnergyDissipation) {
        let scalar_field = rawdata::RawDataScalar::read_raw(eps_path).ok_or(DataError::BadData)?;
        let value: Vec<f64> = scalar_field.values.iter().map(|v| v.value).collect();
        misc.insert(String::from("energy_dissipation"), value);
    }

    let liq_flow_path = case
        .resolve(root, crate::CMAExportType::LiquidFlow)
        .ok_or(DataError::BadData)?;
    let liq_vol_path = case
        .resolve(root, crate::CMAExportType::LiquidVolume)
        .ok_or(DataError::BadData)?;

    let liquid_descriptor = FlowMapDescriptor::from_path(liq_flow_path, liq_vol_path)?;

    let gas_descriptor = match (
        case.resolve(root, crate::CMAExportType::GasFlow),
        case.resolve(root, crate::CMAExportType::GasVolume),
    ) {
        (Some(gas_flow), Some(gas_volume)) => {
            Some(FlowMapDescriptor::from_path(gas_flow, gas_volume)?)
        }
        _ => None,
    };

    Ok(RawStateBuffer(liquid_descriptor, gas_descriptor, misc))
}
