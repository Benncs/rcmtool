// SPDX-License-Identifier: GPL-3.0-or-later


#[allow(clippy::all)]
#[rustfmt::skip]
pub mod generated_domain;
use crate::{CMError, DomainData};
mod reactors;
use reactors::{parse_connection, parse_feed, parse_reactor};

mod pfr_mb;
pub(super) use pfr_mb::PfrGlobalMassBalance;

pub fn parse_domain(
    root: &generated_domain::RootElementType,
) -> Result<(DomainData, PfrGlobalMassBalance), CMError> {
    if root.reactors.content.is_empty() {
        return Err(CMError::Parse(serde_xml_rs::Error::Custom(
            "At least one reactor required".to_owned(),
        )));
    }

    let info = parse_reactor(&root.reactors)?;

    let mut mass_balance = PfrGlobalMassBalance::new(&info.pfr_names);

    let raw_connections = root
        .connections
        .as_ref()
        .map(|connections| parse_connection(&info, connections, &mut mass_balance));

    let mut pfeeds = None;
    if let Some(feeds) = &root.feeds {
        pfeeds = parse_feed(&info, feeds, &mut mass_balance);
    }
    mass_balance.validate()?;
    let run_id = root.run_id.clone();
    Ok((
        DomainData {
            connections: raw_connections,
            info,
            feeds: pfeeds,
            case_path: String::new(),
            run_id,
        },
        mass_balance,
    ))
}

pub fn get_root(content: &str) -> Result<generated_domain::Root, CMError> {
    // let cursor = Cursor::new(content.as_bytes());
    // let mut reader = IoReader::new(cursor).with_error_info();
    // let root = generated_domain::Root::deserialize(&mut reader).unwrap();
    let root = serde_xml_rs::from_str::<generated_domain::Root>(content)?;
    eprintln!("WARNING: Some reactor may miss if xml is not parsed correctly");
    if root.version != 3 {
        panic!("ALED");
    }
    Ok(root)
}
