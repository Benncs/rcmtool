// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum FlowDirection {
    In,
    Out,
}

///Required information to describe a feed
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct FeedFlow {
    pub flow: f64,
    pub position: usize,
    pub output_position: Option<usize>,
}

///Set of feed for each phase and their id
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ParsedFeeds {
    pub liq: HashMap<String, FeedFlow>,
    pub gas: HashMap<String, FeedFlow>,
}

///Basic about domain
//TODO: clean what's should be private or not
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    ///Compartment index offset for given reactor (e.g compartment_cumsum[reactor_id]==10)
    pub(crate) compartment_cumsum: HashMap<String, usize>,
    pub(crate) total_number_compartment: usize,
    ///Reactor id of pfr names, needed to ensure global mass balance
    pub(crate) pfr_names: Vec<String>,
    //TODO improve it
    ///Indicates if case only contains cfd-based reator
    pub(crate) cm_case_only: Option<String>,
    pub(crate) is_two_phase_flow: bool,
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

    pub fn get_total_number_compartment(&self) -> usize {
        self.total_number_compartment
    }
}

///Details about generated domain
//TODO: clean what's should be private or not
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DomainData {
    ///connections between partial flowmaps
    pub(crate) info: DomainInfo,
    ///Information about feed of the resulting merged domain
    pub(crate) feeds: Option<ParsedFeeds>,
    pub(crate) case_path: String,
    pub run_id: String,
}

impl DomainData {
    pub fn feeds(&self) -> &Option<ParsedFeeds> {
        &self.feeds
    }

    pub fn info(&self) -> &DomainInfo {
        &self.info
    }

    pub fn get_case_path(&self) -> impl AsRef<std::path::Path> {
        &self.case_path
    }
}
