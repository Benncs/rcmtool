use cmtool_core::ensight_gold::{self, Geometry};
use std::path::Path;
fn main() {
    let path =
        Path::new("/home/benjamin/Documents/thesis/cfd-cma/Cas_Test_CMA/export/wall_cart.geo");

    let case = cmtool_core::ensight_gold::Case::read(Path::new(
        "/home/benjamin/Documents/thesis/cfd-cma/Cas_Test_CMA/export/wall_cart.encas",
    ));

    // let geo = cmtool_core::CMHandle::init(
    //     [0, 0, 0],
    //     "/home/benjamin/Documents/thesis/cfd-cma/Cas_Test_CMA/export/",
    //     "wall_cart.geo",
    //     cmtool_core::grid::MeshType::Cylindrical,
    // );

    //     println!("{:?}", geo);
}
