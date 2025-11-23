mod data;
mod generators;
mod map_generation;
mod parser;

use std::io;

pub use crate::data::{DomainData, DomainInfo};
use crate::parser::{get_root, parse_domain};
use map_generation::generate_flowmap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CMError {
    #[error("Cmtool encountered an unknown error. Please check the input and try again.")]
    Default,

    #[error("Cmtool error: {0}")]
    Custom(String),

    #[error(
        "Cmtool: Mass balance error in PFR '{0}' with phase {1}. Check your inputs or calculations."
    )]
    MassBalance(String, String),

    #[error("Cmtool data error: {0}. Ensure your data files are correct and accessible.")]
    Data(#[from] cmtool_data::DataError),

    #[error("I/O error: Failed to handle the file '{0}'. Check file path and permissions.")]
    IO(#[from] io::Error),

    #[error(
        "Cmtool parse error: {0}. Verify that the XML input is well-formed and matches expected schema."
    )]
    Parse(#[from] serde_xml_rs::Error),
}

pub fn generate_domain(root_dir: &str, reactor_content: &str) -> Result<DomainData, CMError> {
    let root = get_root(reactor_content)?;
    let domain = parse_domain(&root)?;
    let root_dir = format!("{}/{}", root_dir, root.run_id);
    std::fs::create_dir_all(root_dir.clone())?;
    generate_flowmap(&root_dir, &domain, &root.reactors)?;
    Ok(domain)
}
