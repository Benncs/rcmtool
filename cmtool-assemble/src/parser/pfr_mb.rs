// SPDX-License-Identifier: GPL-3.0-or-later


use std::collections::HashMap;

use cmtool_data::PhaseCM;

use crate::{CMError, data::FlowDirection};

#[derive(Default, Copy, Clone, Debug)]
struct FlowData {
    in_flow: f64,
    out_flow: f64,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PhaseFlow {
    gas: FlowData,
    liquid: FlowData,
}

// Struct to hold flow data for each ID using a HashMap
#[derive(Default, Debug)]
pub struct PfrGlobalMassBalance {
    flows: HashMap<String, PhaseFlow>,
    validated: bool,
}

impl PfrGlobalMassBalance {
    pub(super) fn new(pfr_names: &[String]) -> Self {
        let mut flows = HashMap::new();
        for name in pfr_names {
            flows.insert(name.clone(), PhaseFlow::default());
        }
        PfrGlobalMassBalance {
            flows,
            validated: false,
        }
    }

     pub(super) fn validate(&mut self) -> Result<(), CMError> {
        for (i, flow) in &self.flows {
            if flow.gas.in_flow != flow.gas.out_flow {
                return Err(CMError::MassBalance(i.clone(), "gas".to_owned()));
            }
            if flow.liquid.in_flow != flow.liquid.out_flow {
                return Err(CMError::MassBalance(i.clone(), "liquid".to_owned()));
            }
        }
        self.validated = true;
        Ok(())
    }

    pub(crate) fn update_flow(
        &mut self,
        id: &str,
        phase: PhaseCM,
        direction: FlowDirection,
        vflow: f64,
    ) {
        self.validated = false;
        if let Some(phase_flow) = self.flows.get_mut(id) {
            let flow_data = match phase {
                PhaseCM::Gas => &mut phase_flow.gas,
                PhaseCM::Liquid => &mut phase_flow.liquid,
            };

            match direction {
                FlowDirection::In => flow_data.in_flow += vflow,
                FlowDirection::Out => flow_data.out_flow += vflow,
            }
        }
    }
    pub fn get_flow(&self, id: &str, phase: PhaseCM) -> Result<f64, CMError> {
        if !self.validated {
            return Err(CMError::Custom("Mass balance needs to be validated before being accessed".to_owned())); 
        }

        if let Some(phase_flow) = self.flows.get(id) {
            return Ok(match phase {
                PhaseCM::Gas => phase_flow.gas.in_flow,
                PhaseCM::Liquid => phase_flow.liquid.in_flow,
            });
        }
        Err(CMError::Custom(format!("Mass balance does not provide {} pfr",id)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pfr_mass_balance() {
        let names = ["name1".to_owned(), "name2".to_owned()];
        let mut mb = PfrGlobalMassBalance::new(&names);

        let expected_ok = mb.validate(); //BEcause flow is null
        assert!(expected_ok.is_ok());

        mb.update_flow("name1", PhaseCM::Gas, FlowDirection::In, 4.);
        let expected_err = mb.validate();
        assert!(expected_err.is_err());

        mb.update_flow("name1", PhaseCM::Gas, FlowDirection::Out, 2.);
        mb.update_flow("name1", PhaseCM::Gas, FlowDirection::Out, 2.);

        let expected_ok = mb.validate();
        assert!(expected_ok.is_ok());

        let pfr_flow = mb.get_flow("name1", PhaseCM::Gas).unwrap();
        assert!(pfr_flow == 4.)
    }
}
