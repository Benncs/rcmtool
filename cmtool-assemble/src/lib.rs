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

        //TODO: Do not create all, return error if not root_dir
        // std::fs::create_dir_all(root_path)?;
        let gc = generate_flowmap(None, &root.reactors, &mb, connections)?;

        //A contract is written in root_dir
        if gc.is_some() {
            domain.case_path = root_dir.as_ref().to_string_lossy().to_string();
        }
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::Parser;
    use crate::{
        CMError, DomainData, GenerateContract,
        generators::{Generator, Reactor0DDescriptor},
    };
    use cmtool_data::{CMCaseJson, CMCaseReader, CMCaseWriter, DEFAULT_CASE_FILE_NAME};

    const XML_FROM_FILE: &str = r#"<?xml version="1.0"?>
<Root run_id="from_file" version="3">
  <Reactors>
    <ReactorFromFile id="str">
      <Path>PATH</Path>
    </ReactorFromFile>
  </Reactors>
</Root>"#;

    fn gen_reactor(
        dir: &Path,
        out_dir: &Path,
    ) -> Result<(DomainData, Option<GenerateContract>), CMError> {
        let steady_xml = XML_FROM_FILE.replace("PATH", dir.to_str().unwrap());
        let (_, steady_parser) = Parser::start_parsing(&steady_xml).unwrap();
        Parser::continue_parsing_with_path(steady_parser, out_dir)
    }

    ///Existing cases, recursive or not, are read as ReactorFromFile by the parser
    #[test]
    fn test_reactor_from_file() {
        let root = std::env::temp_dir().join("cmtool_test_reactor_from_file");
        let out_dir = root.join("out");
        let steady_dir = root.join("steady");
        let recursive_dir = root.join("recursive");
        let first_state_dir = recursive_dir.join("i_0");
        let second_state_dir = recursive_dir.join("i_1");
        let _ = std::fs::remove_dir_all(&root);

        //Single compartment case in each directory
        for case_dir in [&steady_dir, &first_state_dir, &second_state_dir] {
            std::fs::create_dir_all(case_dir).unwrap();
            let descriptor = Reactor0DDescriptor::from_fraction(5., 0.).unwrap();
            let case_dir_str = case_dir.to_string_lossy().to_string();
            let case = Generator::new()
                .generate_0d(descriptor, Some(case_dir_str))
                .unwrap();
            CMCaseJson::write_case(case, &case_dir.join(DEFAULT_CASE_FILE_NAME)).unwrap();
        }

        //The recursive case points to its i_* folders
        let first_state_case_path = first_state_dir.join(DEFAULT_CASE_FILE_NAME);
        let mut recursive_case = CMCaseJson::read_case(&first_state_case_path).unwrap();
        recursive_case.toggle_recursive();
        let recursive_case_path = recursive_dir.join(DEFAULT_CASE_FILE_NAME);
        CMCaseJson::write_case(recursive_case, &recursive_case_path).unwrap();

        let (steady_domain, steady_contract) = gen_reactor(&steady_dir, &out_dir).unwrap();

        let (recursive_domain, recursive_contract) = gen_reactor(&recursive_dir, &out_dir).unwrap();

        std::fs::remove_dir_all(&root).unwrap();

        assert!(steady_contract.is_some());
        assert_eq!(steady_domain.info().get_total_number_compartment(), 1);

        assert!(recursive_contract.is_none());
        assert_eq!(recursive_domain.info().get_total_number_compartment(), 1);
        assert_eq!(
            recursive_domain.get_case_path().as_ref(),
            recursive_dir.as_path()
        );
    }
}
