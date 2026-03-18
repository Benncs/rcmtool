// SPDX-License-Identifier: GPL-3.0-or-later

use cmtool::CmtoolError;
use cmtool_data::{RawData, RawDataFlux};
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
            Mode::Auto(autoargs) => auto_main(cfdargs.common, autoargs),

            Mode::Manual(_manual_args) => todo!(),
        },
        AllModes::Xml(xml) => {
            let path = PathBuf::from(out_or_default(xml.out_dir));
            cmtool_assemble::headless_generate(&xml.descriptor_path, path)?;
            Ok(())
        }
    }
}

fn auto_main(common: CommonArgs, autoargs: AutoArgs) -> Result<(), CmtoolError> {
    let stem = Path::new(&autoargs.case_path)
        .file_stem() // Gets "casename" as OsStr
        .and_then(|s| s.to_str())
        .unwrap(); // Converts OsStr to &str

    let root_dir = out_or_default(common.out);

    let case = cmtool_core::ensight_gold::case::Case::read(&autoargs.case_path)?;

    std::fs::create_dir_all(&root_dir).unwrap();

    let handle = cmtool_core::CMHandle::init(
        [common.n_i, common.n_j, common.n_k],
        &case.root,
        &case.geometry_file_path,
        cmtool_core::grid::MeshType::Cylindrical,
    )
    .unwrap();

    handle
        .dump_all(format!("{}/{}", root_dir, stem), &case.root, &case.paths)
        .map_err(CmtoolError::Core)?;

    handle.dump_real_volume(format!("{}/{}/vofL", root_dir, stem))?;

    #[cfg(feature = "use_vtk")]
    handle.write_vtk(format!("{}/{}/cma_case.vtu", root_dir, stem));

    let f = cmtool::check_flows(
        &RawDataFlux::read_raw("./out/cuve_sldmsh_initmrf/velocity.raw").unwrap(),
    )
    .unwrap();
    println!("{}", f);
    // std::fs::write("/tmp/checks.csv", f);

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
