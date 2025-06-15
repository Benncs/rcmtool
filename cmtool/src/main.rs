use std::path::Path;
use cmtool_core::ensight_gold::{self, Geometry};
fn main()
{
     let path = Path::new("/home/benjamin/Documents/thesis/cfd-cma/Cas_Test_CMA/export/wall_cart.geo");
        let mut reader = ensight_gold::Reader::new(path).unwrap();

        let geo = ensight_gold::Geometry::read(&mut reader);

        println!("{:?}",geo);


}