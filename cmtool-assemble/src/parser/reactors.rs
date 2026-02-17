// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::path::PathBuf;

use super::PfrGlobalMassBalance;
use super::generated_domain;
use crate::data::FeedFlow;
use crate::data::ParsedFeeds;
use crate::parser::generated_domain::FeedFluxType;
use crate::{
    CMError,
    data::{DomainInfo, FlowDirection},
};
use cmtool_data::{PhaseCM, RawDataFlux};

fn connection_per_phase(
    info: &DomainInfo,
    mass_balance: &mut PfrGlobalMassBalance,
    phase_node: &[generated_domain::FluxType],
    phase: PhaseCM,
) -> cmtool_data::RawDataFlux {
    let n_node = phase_node.len();

    let mut rd = RawDataFlux::new(info.total_number_compartment, n_node);

    for (node, flux) in phase_node.iter().zip(rd.fluxes.iter_mut()) {
        if let (Some(absolute_source_id), Some(absolute_target_id)) = (
            info.get_relative_compartment_number(&node.source.id, node.source.compartment_id),
            info.get_relative_compartment_number(&node.target.id, node.target.compartment_id),
        ) {
            let flow = f64::from(node.value.content);
            flux.id_source = absolute_source_id as u32;
            flux.id_target = absolute_target_id as u32;
            flux.flux_source_target = flow;
            flux.flux_target_source = 0.; //Connection is pure PlugFLow

            let f_pfr_source = info.pfr_names.contains(&node.source.id);
            let is_same_node =
                absolute_source_id == absolute_target_id && node.source.id == node.target.id;

            if is_same_node && f_pfr_source {
                mass_balance.update_flow(&node.target.id, phase, FlowDirection::Out, flow);
            } else {
                let f_pfr_target = info.pfr_names.contains(&node.target.id);
                if f_pfr_target {
                    mass_balance.update_flow(&node.target.id, phase, FlowDirection::In, flow);
                }
                if f_pfr_source {
                    mass_balance.update_flow(&node.source.id, phase, FlowDirection::Out, flow);
                }
            }
        } else {
            eprintln!("Ignored connection");
        }
    }
    rd
}

pub fn parse_connection(
    info: &DomainInfo,
    connections: &generated_domain::ConnectionsType,
    mass_balance: &mut PfrGlobalMassBalance,
) -> [RawDataFlux; 2] {
    let liquid_connection: Vec<generated_domain::FluxType> = connections
        .flux
        .iter()
        .filter(|f| f.phase == *"liquid")
        .cloned()
        .collect();

    let gas_connection: Vec<generated_domain::FluxType> = connections
        .flux
        .iter()
        .filter(|f| f.phase == *"gas")
        .cloned()
        .collect();

    [
        connection_per_phase(info, mass_balance, &liquid_connection, PhaseCM::Liquid),
        connection_per_phase(info, mass_balance, &gas_connection, PhaseCM::Gas),
    ]
}

// fn convert_feed_flux_to_flux(feed: generated_domain::FeedFluxType) -> generated_domain::FluxType {
//     generated_domain::FluxType {
//         source: feed.source,
//         target: feed.target,
//         phase: feed.phase,
//         value: feed.value,
//     }
// }

// pub fn parse_feed(
//     info: &DomainInfo,
//     feeds: &generated_domain::FeedsType,
//     mass_balance: &mut PfrGlobalMassBalance,
// ) -> Result<(), CMError> {
//     let liquid_feed: Vec<generated_domain::FluxType> = feeds
//         .flux
//         .iter()
//         .filter(|f| f.phase == *"liquid")
//         .map(|feed| convert_feed_flux_to_flux(feed.clone()))
//         .collect();
//     let gas_feed: Vec<generated_domain::FluxType> = feeds
//         .flux
//         .iter()
//         .filter(|f| f.phase == *"gas")
//         .map(|feed| convert_feed_flux_to_flux(feed.clone()))
//         .collect();
//     connection_per_phase(info, mass_balance, &liquid_feed, PhaseCM::Liquid);
//     connection_per_phase(info, mass_balance, &gas_feed, PhaseCM::Gas);
//     Ok(())
// }
fn parse_feed_phase(
    info: &DomainInfo,
    feeds: &[&FeedFluxType],
    phase: PhaseCM,
    mass_balance: &mut PfrGlobalMassBalance,
) -> HashMap<String, FeedFlow> {
    let fluxes: Vec<generated_domain::FluxType> = feeds
        .iter()
        .map(|feed| generated_domain::FluxType {
            source: feed.source.clone(),
            target: feed.target.clone(),
            phase: feed.phase.clone(),
            value: feed.value.clone(),
        })
        .collect();

    let id: Vec<String> = feeds.iter().map(|feed| feed.id.clone()).collect();
    let rd = connection_per_phase(info, mass_balance, &fluxes, phase);

    rd.fluxes
        .iter()
        .zip(id)
        .map(|(flux, id)| {
            (
                id,
                FeedFlow {
                    flow: flux.flux_source_target,
                    position: flux.id_source as usize,
                    output_position: None, //TODO
                },
            )
        })
        .collect()
}
pub fn parse_feed(
    info: &DomainInfo,
    feeds: &generated_domain::FeedsType,
    mass_balance: &mut PfrGlobalMassBalance,
) -> Option<ParsedFeeds> {
    let (liquid_feeds, gas_feeds): (Vec<_>, Vec<_>) = feeds
        .flux
        .iter()
        .partition(|f| f.phase == *"liquid")
        .clone();
    if liquid_feeds.is_empty() && gas_feeds.is_empty() {
        return None;
    }
    Some(ParsedFeeds {
        liq: parse_feed_phase(info, &liquid_feeds, PhaseCM::Liquid, mass_balance),
        gas: parse_feed_phase(info, &gas_feeds, PhaseCM::Gas, mass_balance),
    })
}

pub fn parse_reactor(reactors: &generated_domain::ReactorsType) -> Result<DomainInfo, CMError> {
    let mut domain_info = DomainInfo::default();
    let mut cm_case_only = None;
    let mut in_place_cumsum = 0;

    for reactor in &reactors.content {
        match reactor {
            generated_domain::ReactorsTypeContent::Reactor0D(reactor0_dtype) => {
                domain_info
                    .compartment_cumsum
                    .insert(reactor0_dtype.id.clone(), in_place_cumsum);
                domain_info.total_number_compartment += 1;
                in_place_cumsum += 1;
                domain_info.is_two_phase_flow = reactor0_dtype.volume_fraction.content != 0.;
            }
            generated_domain::ReactorsTypeContent::Reactor1D(current_pfr) => {
                domain_info
                    .compartment_cumsum
                    .insert(current_pfr.id.clone(), in_place_cumsum);
                let n_c = current_pfr.compartments;
                domain_info.total_number_compartment += n_c;
                domain_info.is_two_phase_flow = current_pfr.volume_fraction.content != 0.;
                domain_info.pfr_names.push(current_pfr.id.clone());
                in_place_cumsum += n_c;
            }
            generated_domain::ReactorsTypeContent::ReactorFromFile(reactor_from_file) => {
                let path = format!("{}/cma_case", reactor_from_file.path);

                let path = PathBuf::from(path);
                let case = cmtool_data::read_case(path.as_path())?;
                // case.n_compartment()
                domain_info
                    .compartment_cumsum
                    .insert(reactor_from_file.id.clone(), in_place_cumsum);
                let n_c = case.n_compartment() as usize;
                domain_info.is_two_phase_flow = case.is_two_phase_flow();
                domain_info.total_number_compartment += n_c;
                in_place_cumsum += n_c;

                if cm_case_only.is_none() {
                    cm_case_only = Some(reactor_from_file.path.clone());
                } else {
                    todo!("Existing flowmap merge")
                }
                println!("{:?}", cm_case_only);
            }
            generated_domain::ReactorsTypeContent::Reactor3D(reactor3_dtype) => {
                todo!("{:?}", reactor3_dtype)
            }
        }
    }
    domain_info.cm_case_only = cm_case_only;
    Ok(domain_info)
}
