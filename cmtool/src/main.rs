use std::{env, path::Path};
use clap::Parser;

#[derive(Parser)]
struct GenArgs
{
    case_path:String,
    n_i:usize,
    n_j:usize,
    n_k:usize,
    out:Option<String>
}


fn main() {
    // let case = cmtool_core::ensight_gold::Case::read(
    //     "/home/benjamin/Documents/thesis/cfd-cma/Cas_Test_CMA/export/wall_cart.encas",
    // )
    // .unwrap();

    let args = GenArgs::parse();



      let stem = Path::new(&args.case_path)
    .file_stem()       // Gets "mycase" as OsStr
    .and_then(|s| s.to_str()).unwrap(); // Converts OsStr to &str

    let case = cmtool_core::ensight_gold::Case::read(&args.case_path,
    )
    .unwrap();
    
    let root_dir = args.out.unwrap_or(format!("{}/../out/", env!("CARGO_MANIFEST_DIR")));
    
    std::fs::create_dir_all(&root_dir).unwrap();


    let handle = cmtool_core::CMHandle::init(
        [args.n_i , args.n_j, args.n_k],
        &case.root,
        &case.geometry_file_path,
        cmtool_core::grid::MeshType::Cylindrical,
    )
    .unwrap();



    handle
        .dump_all(
            format!("{}/{}", root_dir,stem),
            &case.root,
            &case.paths,
        )
        .unwrap();
}
