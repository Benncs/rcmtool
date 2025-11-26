// SPDX-License-Identifier: GPL-3.0-or-later

mod data;
mod generators;
mod map_generation;
mod parser;

use std::io;

pub use crate::data::{DomainData, DomainInfo};
use crate::parser::{generated_domain::RootElementType, get_root, parse_domain};
pub use data::{FeedFlow, ParsedFeeds};
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

pub struct Parser(RootElementType);

impl Parser {
    pub fn start_parsing(reactor_content: &str) -> Result<(String, Self), CMError> {
        let root = get_root(reactor_content)?;
        Ok((root.run_id.clone(), Self(root)))
    }
    fn continue_parsing(p: Parser, root_dir: &str) -> Result<DomainData, CMError> {
        // let (domain, mb) = parse_domain(&root)?;
        // let root_dir = format!("{}/{}", root_dir, root.run_id);
        // std::fs::create_dir_all(root_dir.clone())?;
        // generate_flowmap(&root_dir, &domain, &root.reactors, &mb)?;
        let path = format!("{}/{}", root_dir, p.0.run_id);
        // Ok(domain)
        Self::continue_parsing_with_path(p, path.as_str())
    }

    pub fn continue_parsing_with_path(
        Parser(root): Parser,
        root_dir: &str,
    ) -> Result<DomainData, CMError> {
        let (domain, mb) = parse_domain(&root)?;
        let root_dir = format!("{}", root_dir);
        std::fs::create_dir_all(root_dir.clone())?;
        generate_flowmap(&root_dir, &domain, &root.reactors, &mb)?;

        Ok(domain)
    }
}

pub fn generate_domain(root_dir: &str, reactor_content: &str) -> Result<DomainData, CMError> {
    let (_id, root) = Parser::start_parsing(reactor_content)?;

    let domain = Parser::continue_parsing(root, root_dir)?;

    Ok(domain)
}

pub fn headless_generate(
    reactor_input_file_name: &str,
    path: impl AsRef<std::path::Path>,
) -> Result<(), CMError> {
    let contents = std::fs::read_to_string(reactor_input_file_name)?;
    let root_dir = path.as_ref().to_str().expect("path str").to_owned();
    std::fs::create_dir_all(&root_dir)?;
    generate_domain(&root_dir, &contents)?;
    Ok(())
}
