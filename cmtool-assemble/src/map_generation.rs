// SPDX-License-Identifier: GPL-3.0-or-later


use crate::CMError;
use crate::data::DomainData;
use crate::generators::{Generator, PFRDescription};
use crate::parser::generated_domain;
use crate::parser::{PfrGlobalMassBalance,generated_domain::{Reactor0DType}};
use cmtool_data::{CMCaseReader, PhaseCM};
use cmtool_data::{CCMCaseInfo, CMCaseWriter, DataError};

fn get_volume(size: &generated_domain::GeneralSizeType) -> f64 {
    match &size {
        generated_domain::GeneralSizeType::Volume(v) => v.content as f64,
        generated_domain::GeneralSizeType::Dimension(dim) => {
            (dim.length.content as f64)
                * (dim.diameter.content.powf(2.) as f64)
                * std::f64::consts::PI
                / 4.
        }
    }
}

fn _generate_reactor_0d<T: cmtool_data::CMCaseWriter>(
    save_intermediate: bool,
    generator: &mut Generator,
    root: &str,
    ids: &mut Vec<String>,
    reactor0d: &Reactor0DType,
) -> Result<(), CMError> {
    ids.push(reactor0d.id.clone());
    let volume = get_volume(&reactor0d.size);
    let path = format!("{}/{}", root, reactor0d.id);

    let mut opt_path = None;
    if save_intermediate {
        std::fs::create_dir_all(path.clone()).map_err(DataError::IO)?;

        opt_path = Some(path.clone());
    }

    let case = generator.generate_0d_from_fraction(
        volume,
        reactor0d.volume_fraction.content as f64,
        opt_path,
    )?;

    if save_intermediate {
        T::write_case(case, std::path::Path::new(&format!("{}/cma_case", path)))?;
    }

    Ok(())
}

fn _generate_reactor_1d<T: cmtool_data::CMCaseWriter>(
    save_intermediate: bool,
    generator: &mut Generator,
    root: &str,
    ids: &mut Vec<String>,
    current_pfr: &generated_domain::Reactor1DType,
    mb: &PfrGlobalMassBalance,
) -> Result<(), CMError> {
    ids.push(current_pfr.id.clone());
    let path = format!("{}/{}", root, current_pfr.id);

    let mut opt_path = None;
    if save_intermediate {
        std::fs::create_dir_all(path.clone()).map_err(DataError::IO)?;
        opt_path = Some(path.clone());
    }

    match &current_pfr.size {
        generated_domain::GeneralSizeType::Volume(_) => {
            unimplemented!("pfr needs length")
        }
        generated_domain::GeneralSizeType::Dimension(dim) => {
            eprintln!("TODO: PFR GENERATION W/O FLOW RATES");

            let desc = PFRDescription {
                n_compartment: current_pfr.compartments,
                length: dim.length.content.into(),
                diameter: dim.diameter.content.into(),
                liquid_flow:  mb.get_flow(&current_pfr.id, PhaseCM::Liquid)?,
                gas_flow: mb.get_flow(&current_pfr.id, PhaseCM::Gas)?,
                gas_fraction: current_pfr.volume_fraction.content as f64,
                axial_dispersion: 1e-9,
            };

            let case: cmtool_data::CMCase = generator.generate_1d_from_fraction(desc, opt_path)?;
            if save_intermediate {
                T::write_case(case, std::path::Path::new(&format!("{}/cma_case", path)))?;
            }
        }
    };
    Ok(())
}

fn generate_partial_flowmap<T: cmtool_data::CMCaseWriter>(
    save_intermediate: bool,
    generator: &mut Generator,
    root: &str,
    reactors: &generated_domain::ReactorsType,
     mb: &PfrGlobalMassBalance,
) -> Result<Vec<String>, CMError> {
    let mut ids = Vec::with_capacity(reactors.content.len());

    for reactor in &reactors.content {
        match reactor {
            generated_domain::ReactorsTypeContent::Reactor0D(r) => {
                _generate_reactor_0d::<T>(save_intermediate, generator, root, &mut ids, r)?;
            }
            generated_domain::ReactorsTypeContent::Reactor1D(r) => {
                _generate_reactor_1d::<T>(save_intermediate, generator, root, &mut ids, r,mb)?;
            }
            generated_domain::ReactorsTypeContent::Reactor3D(reactor3_dtype) => {
                todo!("{:?}", reactor3_dtype)
            }
        }
    }
    Ok(ids)
}

pub fn generate_flowmap(
    root: &str,
    domain: &DomainData,
    reactors: &generated_domain::ReactorsType,
    mb: &PfrGlobalMassBalance,
) -> Result<(), CMError> {
    let mut generator = Generator::new();
   
    let save_intermediate = reactors.content.len() == 1;

    let _ids =
        generate_partial_flowmap::<CCMCaseInfo>(save_intermediate, &mut generator, root, reactors,mb)?;

    if _ids.len() > 1 {
        if save_intermediate {
            generator.merge(root, &_ids, domain.connections.clone())?;
        } else {
            generator.merge_from_memory(root, domain.connections.clone())?;
        }
    } else if _ids.len() == 1 && save_intermediate {
        let case_path = format!("{}/{}/cma_case", root, _ids[0]);
        let prep = format!("./{}", _ids[0]);
        let case = CCMCaseInfo::read_case(std::path::Path::new(&case_path))?.prepend_path(&prep);

        CCMCaseInfo::write_case(case, std::path::Path::new(&format!("{}/cma_case", root)))?;
    }

    Ok(())
}

// fn parse_generate()
// {}
