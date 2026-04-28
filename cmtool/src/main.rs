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
            Mode::Auto(autoargs) => {
                if let Err(e) = auto_main(cfdargs.common, autoargs) {
                    eprintln!("{}", e);
                    return Err(e);
                }
                Ok(())
            }

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
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap();

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
