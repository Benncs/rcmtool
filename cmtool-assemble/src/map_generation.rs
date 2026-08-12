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

///The flow map of a ReactorFromFile is not generated, it is read back from the case it points to
fn _generate_reactor_from_file(
    generator: &mut Generator,
    root: &Option<impl AsRef<std::path::Path>>,
    ids: &mut Vec<String>,
    reactor: &generated_domain::ReactorFromFileType,
) -> Result<(), CMError> {
    ids.push(reactor.id.clone());
    generator.add_existing_case(&reactor.id, &reactor.path, root.is_none())
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
            generated_domain::ReactorsTypeContent::ReactorFromFile(r) => {
                _generate_reactor_from_file(generator, &root, &mut ids, r)?;
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

#[cfg(test)]
mod test {
    use cmtool_data::{CMAExportType, CMCaseJson, CMCaseReader, CMCaseWriter, RawData};

    ///Existing case of a single liquid compartment, as a ReactorFromFile would point to
    fn write_existing_case(path: &str) {
        std::fs::create_dir_all(path).unwrap();
        let case = crate::generators::Generator::new()
            .generate_0d(
                crate::generators::Reactor0DDescriptor::from_fraction(10., 0.),
                Some(path.to_owned()),
            )
            .expect("existing case");
        CMCaseJson::write_case(
            case,
            &std::path::Path::new(path).join(cmtool_data::DEFAULT_CASE_FILE_NAME),
        )
        .expect("existing cma_case");
    }

    fn domain_xml(existing_path: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<Root run_id="from_file" version="3">
  <Reactors>
    <ReactorFromFile id="existing">
      <Path>{existing_path}</Path>
    </ReactorFromFile>
    <Reactor0D id="str">
      <Size>
        <Volume>5</Volume>
      </Size>
      <VolumeFraction>0</VolumeFraction>
    </Reactor0D>
  </Reactors>
  <Connections>
    <Flux phase="liquid">
      <Source id="existing" compartment_id="0"></Source>
      <Target id="str" compartment_id="0"></Target>
      <Value unit="l/min">2</Value>
    </Flux>
  </Connections>
</Root>"#
        )
    }

    ///A ReactorFromFile is a reactor like the others: its existing flow map is merged with the
    ///generated ones instead of replacing the whole domain
    #[test]
    fn test_merge_reactor_from_file_with_generated_reactor() {
        let root = "/tmp/test_reactor_from_file";
        let existing = format!("{}/existing", root);
        let _ = std::fs::remove_dir_all(root);
        write_existing_case(&existing);

        let domain =
            crate::generate_and_write_domain(root, &domain_xml(&existing)).expect("domain");

        let merged = CMCaseJson::read_case(std::path::Path::new(root).join("cma_case").as_path())
            .expect("merged case");
        let flow_path = merged
            .resolve(root, CMAExportType::LiquidFlow)
            .expect("flow");
        let volume_path = merged
            .resolve(root, CMAExportType::LiquidVolume)
            .expect("volume");

        let flow = cmtool_data::RawDataFlux::read_raw(flow_path).expect("liquid flow");
        let volume = cmtool_data::RawDataScalar::read_raw(volume_path).expect("liquid volume");

        //The existing compartment comes first, the generated one is offset behind it
        assert_eq!(flow.header.n_zone, 2);
        assert_eq!(volume.values.len(), 2);
        assert_eq!(volume.values[0].value, 10.);
        assert_eq!(volume.values[1].value, 5.);

        //The connection between both reactors survived the merge
        assert!(
            flow.fluxes
                .iter()
                .any(|f| f.id_source == 0 && f.id_target == 1 && f.flux_source_target == 2.)
        );
        assert_eq!(domain.info().total_number_compartment, 2);

        std::fs::remove_dir_all(root).unwrap();
    }
}

// fn parse_generate()
// {}
