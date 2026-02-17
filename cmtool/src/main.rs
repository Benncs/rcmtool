// SPDX-License-Identifier: GPL-3.0-or-later

use cmtool::CmtoolError;
use std::fmt::Write;

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

            Mode::Manual(manual_args) => todo!(),
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

    #[cfg(feature = "use_vtk")]
    handle.write_vtk(format!("{}/{}/cma_case.vtu", root_dir, stem));
    return Ok(());
    let p1 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl1";
    let p2 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl2";
    let p3 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl3";
    //
    let export = handle
        .dump_vector_from_scalar(format!("{}/flowL", root_dir), p1, p2, p3)
        .unwrap();

    if let Some(mut check_csv) = cmtool::check_flows(&export) {
        check_csv
            .write_str(&format!("sanitize_{}.csv", stem))
            .unwrap();
    };

    Ok(())

    //  cmtool::check_flows(&RawDataFlux::read_raw(
    //      "./out/cuve_sldmsh_initmrf/axial_velocity.raw",
    //  ).unwrap());

    //    cmtool::check_flows(&RawDataFlux::read_raw(
    //         "/home/benjamin/Documents/thesis/cfd-cma/sanofi/raw/flowL.raw",
    //     ).unwrap());
}

// fn main() {
//     // #[cfg(debug_assertions)]
//     let args = GenArgs {
//         case_path: "/home/benjamin/Documents/thesis/cfd-cma/rushton/cuve_sldmsh_initmrf.encas"
//             .to_string(),
//         n_i: 10,
//         n_j: 10,
//         n_k: 5,
//         out: None,
//     };

//     //#[cfg(not(debug_assertions))]
//     //let args = GenArgs::parse();

//     let stem = Path::new(&args.case_path)
//         .file_stem() // Gets "mycase" as OsStr
//         .and_then(|s| s.to_str())
//         .unwrap(); // Converts OsStr to &str

//     let case = cmtool_core::ensight_gold::Case::read(&args.case_path).unwrap();

//     let root_dir = args
//         .out
//         .unwrap_or(format!("{}/../out/", env!("CARGO_MANIFEST_DIR")));

//     std::fs::create_dir_all(&root_dir).unwrap();

//     let handle = cmtool_core::CMHandle::init(
//         [args.n_i, args.n_j, args.n_k],
//         &case.root,
//         &case.geometry_file_path,
//         cmtool_core::grid::MeshType::Cylindrical,
//     )
//     .unwrap();

//     handle
//         .dump_all(format!("{}/{}", root_dir, stem), &case.root, &case.paths)
//         .unwrap();
//     handle
//         .dump_real_volume(format!("{}/{}/total_volume", root_dir, stem))
//         .unwrap();
//     //     let p1 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl1";
//     //     let p2 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl2";
//     //     let p3 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl3";
//     // //
//     //     let export = handle
//     //         .dump_vector_from_scalar(format!("{}/flowL", root_dir), p1, p2, p3)
//     //         .unwrap();

//     //     cmtool::check_flows(&export);

//     cmtool::check_flows(&RawDataFlux::read_raw("./out/cuve_sldmsh_initmrf/velocity.raw").unwrap());
// }
