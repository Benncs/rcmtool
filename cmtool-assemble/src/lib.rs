// SPDX-License-Identifier: GPL-3.0-or-later

mod data;
mod errors;
mod generators;
mod map_generation;
mod parser;

pub use crate::data::{DomainData, DomainInfo};
use crate::parser::{generated_domain::RootElementType, get_root, parse_domain};
pub use cmtool_data::PhaseCM; //reexport to have easier dependency
pub use data::{FeedFlow, ParsedFeeds};
pub use errors::CMError;
pub use generators::GenerateContract;
use map_generation::generate_flowmap;
pub type ConnectionType = [cmtool_data::RawDataFlux; 2];

/// Domain parser
// TODO: make it private + change name
pub struct Parser(RootElementType);

impl Parser {
    /// Create parser from xml content
    /// Returns id + object if suceeds
    pub fn start_parsing(reactor_content: &str) -> Result<(String, Self), CMError> {
        let root = get_root(reactor_content)?;
        Ok((root.run_id.clone(), Self(root)))
    }

    /// Parse domain and generate content in the root directory if needed
    /// Returns info about generated domain if suceeds
    fn continue_parsing(
        p: Parser,
        root_dir: impl AsRef<std::path::Path>,
    ) -> Result<(DomainData, Option<GenerateContract>), CMError> {
        let path = root_dir.as_ref().join(&p.0.run_id);
        Self::continue_parsing_with_path(p, path)
    }

    /// Parse domain and generate content at given abolute path if needed
    /// Returns info about generated domain if suceeds
    pub fn continue_parsing_with_path(
        Parser(root): Parser,
        root_dir: impl AsRef<std::path::Path>,
    ) -> Result<(DomainData, Option<GenerateContract>), CMError> {
        let (mut domain, mb, connections) = parse_domain(&root)?;

        let (path, gc): (std::path::PathBuf, Option<GenerateContract>) =
            if let Some(cm_case) = &domain.info().cm_case_only {
                (std::path::PathBuf::from(&cm_case), None)
            } else {
                //TODO: Do not create all, return error if not root_dir
                // std::fs::create_dir_all(root_path)?;
                let gc = generate_flowmap(None, &root.reactors, &mb, connections)?;
                (root_dir.as_ref().to_owned(), gc)
            };

        domain.case_path = path.as_os_str().to_string_lossy().to_string();
        Ok((domain, gc))
    }
}

//Parse and generate domain at root dir  from give xml content
// Returns info about domain if suceeds
pub fn generate_domain(
    root_dir: impl AsRef<std::path::Path>,
    reactor_content: &str,
) -> Result<(DomainData, Option<GenerateContract>), CMError> {
    let (_id, parser) = Parser::start_parsing(reactor_content)?;

    Parser::continue_parsing(parser, root_dir)
}

//Parse and generate domain at root dir  from give xml content
// Returns info about domain if suceeds
pub fn generate_and_write_domain(
    root_dir: impl AsRef<std::path::Path>,
    reactor_content: &str,
) -> Result<DomainData, CMError> {
    let (domain, gc) = generate_domain(&root_dir, reactor_content)?;
    if let Some(contract) = gc {
        contract.write(root_dir)?;
    }
    Ok(domain)
}

//Parse and generate domain at root dir  from give xml filepath
// For headless/cli use reads/create folder and generate
pub fn headless_generate(
    reactor_input_file_name: &str,
    path: impl AsRef<std::path::Path>,
) -> Result<(), CMError> {
    let contents = std::fs::read_to_string(reactor_input_file_name)?;
    //Useless because already created in continue_parsing_with_path
    //TODO: keep it here and returns errors if not exist
    let root_dir = path.as_ref().to_str().expect("path str").to_owned();
    std::fs::create_dir_all(&root_dir)?;
    generate_domain(&root_dir, &contents)?;
    Ok(())
}
