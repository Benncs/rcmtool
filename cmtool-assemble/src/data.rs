// SPDX-License-Identifier: GPL-3.0-or-later


use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum FlowDirection {
    In,
    Out,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct FeedFlow {
    pub flow: f64,
    pub position: usize,
    pub output_position: Option<usize>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ParsedFeeds {
    pub liq: HashMap<String, FeedFlow>,
    pub gas: HashMap<String, FeedFlow>,
}
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub compartment_cumsum: HashMap<String, usize>,
    pub total_number_compartment: usize,
    pub pfr_names: Vec<String>,
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

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DomainData {
    pub connections: Option<[cmtool_data::RawDataFlux; 2]>,
    pub info: DomainInfo,
    pub feeds: Option<ParsedFeeds>,
}





