// SPDX-License-Identifier: GPL-3.0-or-later

use cmtool::CmtoolError;
use cmtool_core::model::BalanceSettings;
use cmtool_data::{
    CMAExportType, CMCase, CMCaseJson, CMCaseWriter, DEFAULT_CASE_FILE_NAME, RawData, RawDataFlux,
    RawDataScalar,
};
// use std::fmt::Write;

// use std::fs;
use std::path::PathBuf;
use std::{env, path::Path};
mod args;
use args::*;

fn out_or_default(out: Option<String>) -> String {
    out.unwrap_or(format!("{}/../out/", env!("CARGO_MANIFEST_DIR")))
}

fn main() -> Result<(), CmtoolError> {
    let args = GenArgs::get();
    let mode = args.mode;
    match mode {
        AllModes::Cfd(cfdargs) => match cfdargs.mode {
            Mode::Auto(autoargs) => {
                if let Err(e) = auto_main(cfdargs.common, autoargs) {
                    eprintln!("{}", e);
                    return Err(e);
                }
                Ok(())
            }

            Mode::Manual(manual_args) => manual_main(cfdargs.common, manual_args),
        },
        AllModes::Xml(xml) => {
            let path = PathBuf::from(out_or_default(xml.out_dir));
            cmtool_assemble::headless_generate(&xml.descriptor_path, path)?;
            Ok(())
        }
    }
}

///Balancing knobs, the command line only overrides what it was given
fn balance_settings(common: &CommonArgs, handle: &cmtool_core::CMHandle) -> BalanceSettings {
    let mut balance = *handle.balance_settings();
    if let Some(tolerance) = common.balance_tolerance {
        balance.tolerance = tolerance;
    }
    if let Some(iterations) = common.balance_iterations {
        balance.max_iterations = iterations;
    }
    if let Some(max_divergence) = common.max_divergence {
        balance.max_divergence = max_divergence;
    }
    balance
}

fn three_paths(root: &str, files: &[String]) -> Option<[PathBuf; 3]> {
    match files {
        [i, j, k] => Some([
            Path::new(root).join(i),
            Path::new(root).join(j),
            Path::new(root).join(k),
        ]),
        _ => None,
    }
}

///Builds a case out of velocities given as separate scalar components, which is how a CFD export
///stores them when it holds no vector variable
fn manual_main(common: CommonArgs, args: ManualArgs) -> Result<(), CmtoolError> {
    if args.liquid.is_empty()
        && args.gas.is_empty()
        && args.vectors.is_empty()
        && args.scalars.is_empty()
    {
        return Err(CmtoolError::Custom(String::from(
            "Nothing to generate: give a scalar, a vector, or the three velocity components of a phase",
        )));
    }

    let root_dir = out_or_default(common.out.clone());
    std::fs::create_dir_all(&root_dir).map_err(cmtool_data::DataError::IO)?;

    let mut handle = cmtool_core::CMHandle::init(
        [common.n_i, common.n_j, common.n_k],
        &args.root,
        &args.geo_file,
        cmtool_core::grid::MeshType::Cylindrical,
    )?;
    handle.set_balance_settings(balance_settings(&common, &handle));

    let scalar_path = |name: &str| Path::new(&args.root).join(name);
    let out_path = |name: &str| format!("{}/{}", root_dir, name);

    //The gas fraction splits the phases: a phase carries its share of the flow and of the volume
    let gas_fraction = args
        .gas_fraction
        .as_ref()
        .map(|name| handle.get_scalar(scalar_path(name)))
        .transpose()?;

    let mut has_flow_map = false;
    let mut case = CMCase::new(
        [common.n_i as u32, common.n_j as u32, common.n_k as u32],
        0.,
        Some(String::from("generated from scalar components")),
        false,
    );

    if let Some(liquid) = three_paths(&args.root, &args.liquid) {
        let fraction = args
            .gas_fraction
            .as_ref()
            .map(|name| {
                handle
                    .get_scalar(scalar_path(name))
                    .map(|s| s.scalar_shift(1.))
            })
            .transpose()?;
        handle.dump_vector_from_scalar(
            out_path("flowL"),
            &liquid[0],
            &liquid[1],
            &liquid[2],
            fraction,
        )?;
        case.add(CMAExportType::LiquidFlow, "flowL.raw");
        has_flow_map = true;
    }

    if let Some(gas) = three_paths(&args.root, &args.gas) {
        handle.dump_vector_from_scalar(
            out_path("flowG"),
            &gas[0],
            &gas[1],
            &gas[2],
            gas_fraction,
        )?;
        case.add(CMAExportType::GasFlow, "flowG.raw");
    }

    //Integrating the gas fraction over a compartment gives the volume the gas occupies in it
    let total_volume = handle.real_volume();
    let gas_volume = match &args.gas_fraction {
        Some(name) => handle
            .dump_scalar(out_path("vofG"), scalar_path(name))?
            .values
            .iter()
            .map(|v| v.value)
            .collect(),
        None => vec![0.; total_volume.len()],
    };

    let liquid_volume: Vec<f64> = total_volume
        .iter()
        .zip(&gas_volume)
        .map(|(total, gas)| total - gas)
        .collect();

    RawDataScalar::from(liquid_volume.as_slice()).write_raw(&out_path("vofL.raw"))?;
    case.add(CMAExportType::LiquidVolume, "vofL.raw");
    if args.gas_fraction.is_some() {
        case.add(CMAExportType::GasVolume, "vofG.raw");
    }

    //A scalar is integrated over each compartment, a vector becomes a flow map of its own
    let file_name = |path: &str| {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_owned()
    };

    for scalar in &args.scalars {
        handle.dump_scalar(out_path(&file_name(scalar)), scalar_path(scalar))?;
    }

    for vector in &args.vectors {
        handle.dump_vector(out_path(&file_name(vector)), scalar_path(vector))?;
    }

    //A case needs a flow map to go with its volumes, dumping scalars alone does not make one
    if has_flow_map {
        CMCaseJson::write_case(case, &Path::new(&root_dir).join(DEFAULT_CASE_FILE_NAME))?;
        println!("Case written in {}", root_dir);
    } else {
        println!(
            "Files written in {}, no case: it holds no flow map",
            root_dir
        );
    }

    Ok(())
}

fn auto_main(common: CommonArgs, autoargs: AutoArgs) -> Result<(), CmtoolError> {
    let stem = Path::new(&autoargs.case_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap();

    let root_dir = out_or_default(common.out);

    let case = cmtool_core::ensight_gold::case::Case::read(&autoargs.case_path)?;

    std::fs::create_dir_all(&root_dir).unwrap();

    let mut handle = cmtool_core::CMHandle::init(
        [common.n_i, common.n_j, common.n_k],
        &case.root,
        &case.geometry_file_path,
        cmtool_core::grid::MeshType::Cylindrical,
    )
    .unwrap();

    //Flow balancing keeps its defaults unless the command line says otherwise
    let mut balance = *handle.balance_settings();
    if let Some(tolerance) = common.balance_tolerance {
        balance.tolerance = tolerance;
    }
    if let Some(iterations) = common.balance_iterations {
        balance.max_iterations = iterations;
    }
    if let Some(max_divergence) = common.max_divergence {
        balance.max_divergence = max_divergence;
    }
    handle.set_balance_settings(balance);

    handle
        .dump_all(format!("{}/{}", root_dir, stem), &case.root, &case.paths)
        .map_err(CmtoolError::Core)?;

    handle.dump_real_volume(format!("{}/{}/vofL", root_dir, stem))?;
    handle.dump_real_volume(format!("{}/{}/vtot", root_dir, stem))?;

    // let path_gas_f = case.paths.iter().find(|f| f.name == "gas_vof").unwrap();

    // let liquid_fraction = handle
    //     .get_scalar(std::path::PathBuf::from(&case.root).join(&path_gas_f.filepath))
    //     .unwrap()
    //     .scalar_shift(1.);

    // let manual_flowl = handle
    //     .vector_from_scalar(
    //         "/tmp/inputs/RESULTS.scl1",
    //         "/tmp/inputs/RESULTS.scl2",
    //         "/tmp/inputs/RESULTS.scl3",
    //     )
    //     .unwrap()
    //     .scale_by(liquid_fraction)
    //     .unwrap();

    // handle.dump_vector_raw(format!("{}/{}/flowL", root_dir, stem), manual_flowl)?;

    // handle.dump_vector_from_scalar(
    //     format!("{}/{}/flowL", root_dir, stem),
    // "/tmp/sanofi/inputs/RESULTS.scl1",
    // "/tmp/sanofi/inputs/RESULTS.scl2",
    // "/tmp/sanofi/inputs/RESULTS.scl3",
    // )?;

    // #[cfg(feature = "use_vtk")]
    // handle.write_vtk(format!("{}/{}/cma_case.vtu", root_dir, stem));

    let _f = cmtool::check_flows(
        handle.grid(),
        &RawDataFlux::read_raw("./out/RESULTS/flowL.raw").unwrap(),
    )
    .unwrap();
    // println!("{}", f);
    // std::fs::write("/tmp/checks.csv", f);
    //
    // let f = cmtool::divergence_free(
    //     &RawDataFlux::read_raw("./out/cuve_sldmsh_initmrf/velocity.raw").unwrap(),
    // );
    // f.write_raw("./out/cuve_sldmsh_initmrf/velocity2.raw")
    //     .unwrap();
    // let mut case = cmtool_data::CMCase::new(
    //     [common.n_i as u32, common.n_j as u32, common.n_k as u32],
    //     0.,
    //     None,
    //     false,
    // );

    // cmtool_data::CMCaseJson::write_case(
    //     case,
    //     std::path::Path::new(&format!("{}/{}/cma_case", root_dir, stem)),
    // )?;

    Ok(())
}
