
fn resolve_path(root:&str,relative_path:&str)->impl AsRef<std::path::Path>{
    format!("{}/{}",root,relative_path)
}

fn main() {
    // let case = cmtool_core::ensight_gold::Case::read(
    //     "/home/benjamin/Documents/thesis/cfd-cma/Cas_Test_CMA/export/wall_cart.encas",
    // )
    // .unwrap();
    let case = cmtool_core::ensight_gold::Case::read(
        "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.encas",
    )
    .unwrap();

    let geo = cmtool_core::CMHandle::init(
        [3, 3, 3],
        &case.root,
        &case.geometry_file_path,
        cmtool_core::grid::MeshType::Cylindrical,
    ).unwrap();

    geo.dump_scalar(resolve_path(&case.root, &case.paths[8].filepath)).unwrap();

    //     println!("{:?}", geo);
}
