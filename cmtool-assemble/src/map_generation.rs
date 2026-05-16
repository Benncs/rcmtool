// SPDX-License-Identifier: GPL-3.0-or-later

use crate::generators::{Generator, PFRDescription};
use crate::parser::generated_domain::{self, GeneralSizeType};
use crate::parser::{PfrGlobalMassBalance, generated_domain::Reactor0DType};
use crate::{CMError, GenerateContract};
use cmtool_data::DataError;
use cmtool_data::{CMCaseJson, DEFAULT_CASE_FILE_NAME, PhaseCM, RawDataFlux};

impl GeneralSizeType {
    ///Returns volume of reactor considering cylindrical shape
    pub fn get_volume(&self) -> f64 {
        match self {
            generated_domain::GeneralSizeType::Volume(v) => v.content as f64,
            generated_domain::GeneralSizeType::Dimension(dim) => {
                (dim.length.content as f64)
                    * (dim.diameter.content.powf(2.) as f64)
                    * std::f64::consts::PI
                    / 4.
            }
        }
    }
}

fn _generate_reactor_0d<T: cmtool_data::CMCaseWriter>(
    generator: &mut Generator,
    root: &Option<impl AsRef<std::path::Path>>,
    ids: &mut Vec<String>,
    reactor0d: &Reactor0DType,
) -> Result<(), CMError> {
    ids.push(reactor0d.id.clone());
    let volume = reactor0d.size.get_volume();

    let path = root.as_ref().map(|r| r.as_ref().join(reactor0d.id.clone()));

    let mut opt_path = None;
    if let Some(p) = &path {
        std::fs::create_dir_all(p).map_err(DataError::IO)?;
        opt_path = Some(p.to_string_lossy().to_string());
    }

    let descriptor = crate::generators::Reactor0DDescriptor::from_fraction(
        volume,
        reactor0d.volume_fraction.content as f64,
    );

    let case = generator.generate_0d(descriptor, opt_path)?;

    // let case = generator.generate_0d_from_fraction(
    //     volume,
    //     reactor0d.volume_fraction.content as f64,
    //     opt_path,
    // )?;
    if let Some(p) = &path {
        T::write_case(case, &p.join(DEFAULT_CASE_FILE_NAME))?;
    }

    Ok(())
}

fn _generate_reactor_1d<T: cmtool_data::CMCaseWriter>(
    generator: &mut Generator,
    root: &Option<impl AsRef<std::path::Path>>,
    ids: &mut Vec<String>,
    current_pfr: &generated_domain::Reactor1DType,
    mb: &PfrGlobalMassBalance,
) -> Result<(), CMError> {
    ids.push(current_pfr.id.clone());

    let path = root
        .as_ref()
        .map(|r| r.as_ref().join(current_pfr.id.clone()));

    let mut opt_path = None;
    if let Some(p) = &path {
        std::fs::create_dir_all(p).map_err(DataError::IO)?;
        opt_path = Some(p.to_string_lossy().to_string());
    }

    match &current_pfr.size {
        generated_domain::GeneralSizeType::Volume(_) => {
            unimplemented!("pfr needs length")
        }
        generated_domain::GeneralSizeType::Dimension(dim) => {
            eprintln!("TODO: PFR GENERATION W/O FLOW RATES");
            let desc = PFRDescription::new(
                current_pfr.compartments.get(),
                dim.length.content,
                dim.diameter.content,
                mb.get_flow(&current_pfr.id, PhaseCM::Liquid)?,
                mb.get_flow(&current_pfr.id, PhaseCM::Gas)?,
                current_pfr.volume_fraction.content,
                1e-9,
            )
            .map_err(|e| CMError::Custom(format!("Invalid PFR descritor {}", e)))?;

            let case: cmtool_data::CMCase = generator.generate_1d(desc, opt_path)?;
            if let Some(p) = &path {
                // T::write_case(case, std::path::Path::new(&format!("{}/cma_case", p)))?;
                T::write_case(case, &p.join(DEFAULT_CASE_FILE_NAME))?;
            }
        }
    };
    Ok(())
}

fn generate_partial_flowmap<T: cmtool_data::CMCaseWriter>(
    generator: &mut Generator,
    root: Option<impl AsRef<std::path::Path>>,
    reactors: &generated_domain::ReactorsType,
    mb: &PfrGlobalMassBalance,
) -> Result<Vec<String>, CMError> {
    let mut ids = Vec::with_capacity(reactors.content.len());
    for reactor in &reactors.content {
        match reactor {
            generated_domain::ReactorsTypeContent::Reactor0D(r) => {
                _generate_reactor_0d::<T>(generator, &root, &mut ids, r)?;
            }
            generated_domain::ReactorsTypeContent::Reactor1D(r) => {
                _generate_reactor_1d::<T>(generator, &root, &mut ids, r, mb)?;
            }
            generated_domain::ReactorsTypeContent::Reactor3D(reactor3_dtype) => {
                todo!("{:?}", reactor3_dtype)
            }
            generated_domain::ReactorsTypeContent::ReactorFromFile(_) => {
                todo!("generate_partial_flowmap::ReactorFromFile")
            }
        }
    }
    Ok(ids)
}

pub fn generate_flowmap(
    root: Option<std::path::PathBuf>,
    reactors: &generated_domain::ReactorsType,
    mb: &PfrGlobalMassBalance,
    connections: Option<[RawDataFlux; 2]>,
) -> Result<Option<GenerateContract>, CMError> {
    let mut generator = Generator::new();

    let save_intermediate = root.is_some();
    //root is used only if save_intermediae is true
    let _ids = generate_partial_flowmap::<CMCaseJson>(&mut generator, root.clone(), reactors, mb)?;
    //TODO remove returning cm_path  merge and merge_from_memory dont need to return it cause it is 'root'
    //Same when len(id)==1
    // if _ids.len() > 1 {
    //     let gc = if save_intermediate {
    //         generator.merge(root.clone().unwrap(), &_ids, connections.clone())?;
    //         None
    //     } else {
    //         Some(generator.merge_from_memory(connections.clone())?)
    //     };
    //     Ok((root.clone().unwrap().to_str().unwrap().to_owned(), gc))
    // } else if _ids.len() == 1 && save_intermediate {
    //     let r = root.unwrap();
    //     let case_path = r.clone().join(&_ids[0]);

    //     let prep = format!("./{}", _ids[0]);
    //     let case = CMCaseJson::read_case(&case_path)?.prepend_path(&prep);
    //     let path = r.join("cma_case");
    //     CMCaseJson::write_case(case, &path)?;
    //     Ok((r.to_str().unwrap().to_owned(), None))
    // } else {
    //     Err(CMError::Custom("TODO ".to_owned()))
    // }
    if _ids.is_empty() {
        return Err(CMError::Custom("No flowmap to generate".to_owned()));
    }
    let gc = if save_intermediate {
        generator.merge(root.clone().unwrap(), &_ids, connections.clone())?;
        None
    } else {
        Some(generator.merge_from_memory(connections.clone())?)
    };
    Ok(gc)
}

// fn parse_generate()
// {}
