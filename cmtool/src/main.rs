use clap::{Parser, Subcommand};
use cmtool::CmtoolError;
use std::fmt::Write;

use std::{env, path::Path};
#[derive(Parser, Clone)]
struct CommonArgs {
    n_i: usize,
    n_j: usize,
    n_k: usize,
    /// Verbosity level
    #[clap(short, long)]
    verbose: bool,

    /// Output directory
    #[clap(short, long)]
    out: Option<String>,
}

#[derive(Parser, Default, Clone)]
struct ManualArgs {
    root: String,
    geo_file: String,
    scalars: Vec<String>,
    vectors: Vec<String>,
}

#[derive(Parser, Default, Clone)]
struct AutoArgs {
    case_path: String,
}

#[derive(Subcommand, Clone)]
enum Mode {
    Manual(ManualArgs),
    Auto(AutoArgs),
}

#[derive(Parser, Clone)]
#[clap(name = "myapp")]
struct GenArgs {
    #[clap(flatten)]
    common: CommonArgs,

    #[clap(subcommand)]
    mode: Mode,
}

impl GenArgs {
    #[cfg(debug_assertions)]
    fn get() -> Self {
        GenArgs {
            common: CommonArgs {
                n_i: 3,
                n_j: 3,
                n_k: 3,
                out: None,
                verbose: true,
            },
            mode: Mode::Auto(AutoArgs {
                case_path:
                    "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.encas"
                        .to_string(),
            }),
        }
    }
    #[cfg(not(debug_assertions))]
    fn get() -> Self {
        GenArgs::parse()
    }
}


fn main() -> Result<(), CmtoolError> {
    let args = GenArgs::get();
    let mode = args.mode;
    match mode {
        Mode::Auto(autoargs) => auto_main(args.common, autoargs),

        Mode::Manual(manual_args) => todo!(),
    }
}

fn auto_main(common: CommonArgs, autoargs: AutoArgs) -> Result<(), CmtoolError> {
    let stem = Path::new(&autoargs.case_path)
        .file_stem() // Gets "mycase" as OsStr
        .and_then(|s| s.to_str())
        .unwrap(); // Converts OsStr to &str

    let root_dir = common
        .out
        .unwrap_or(format!("{}/../out/", env!("CARGO_MANIFEST_DIR")));

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
