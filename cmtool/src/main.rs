use clap::Parser;
use cmtool_data::{RawData, RawDataFlux};
use std::{env, path::Path};

#[derive(Parser)]
struct GenArgs {
    case_path: String,
    n_i: usize,
    n_j: usize,
    n_k: usize,
    out: Option<String>,
}

fn main() {
    

    #[cfg(debug_assertions)]
    let args = GenArgs {
        case_path: "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.encas"
            .to_string(),
        n_i: 3,
        n_j: 3,
        n_k: 3,
        out: None,
    };

    #[cfg(not(debug_assertions))]
    let args = GenArgs::parse();

    let stem = Path::new(&args.case_path)
        .file_stem() // Gets "mycase" as OsStr
        .and_then(|s| s.to_str())
        .unwrap(); // Converts OsStr to &str

    let case = cmtool_core::ensight_gold::Case::read(&args.case_path).unwrap();

    let root_dir = args
        .out
        .unwrap_or(format!("{}/../out/", env!("CARGO_MANIFEST_DIR")));

    std::fs::create_dir_all(&root_dir).unwrap();

    let handle = cmtool_core::CMHandle::init(
        [args.n_i, args.n_j, args.n_k],
        &case.root,
        &case.geometry_file_path,
        cmtool_core::grid::MeshType::Cylindrical,
    )
    .unwrap();

    handle
        .dump_all(format!("{}/{}", root_dir, stem), &case.root, &case.paths)
        .unwrap();

    //let p1 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl1";
    //let p2 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl2";
    //let p3 = "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.scl3";
//
    //let export = handle
        //.dump_vector_from_scalar(format!("{}/flowL", root_dir), p1, p2, p3)
        //.unwrap();
//
    //cmtool::check_flows(&export);


     cmtool::check_flows(&RawDataFlux::read_raw(
         "./out/cuve_sldmsh_initmrf/axial_velocity.raw",
     ).unwrap());

//    cmtool::check_flows(&RawDataFlux::read_raw(
//         "/home/benjamin/Documents/thesis/cfd-cma/sanofi/raw/flowL.raw",
//     ).unwrap()); 
}
