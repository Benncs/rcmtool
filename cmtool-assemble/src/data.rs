use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub compartment_cumsum: HashMap<String, usize>,
    pub total_number_compartment: usize,
    pub pfr_names: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DomainData {
    pub connections: Option<[cmtool_data::RawDataFlux; 2]>,
    pub info: DomainInfo,
}



impl DomainInfo {
    pub fn get_relative_compartment_number(
        &self,
        reactor_id: &str,
        relative_index: usize,
    ) -> Option<usize> {
        self.compartment_cumsum
            .get(reactor_id)
            .map(|cum_sum| relative_index + *cum_sum)
    }
}

#[derive(Debug, PartialEq)]
pub enum FlowDirection {
    In,
    Out,
}
