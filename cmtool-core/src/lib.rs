pub 
mod utils;
pub mod ensight_gold;
pub mod grid;

mod test
{
    use std::path::Path;

    use super::*;
   
    #[test]
    fn test_read()
    {
        let path = Path::new("/home/benjamin/Documents/thesis/cfd-cma/Cas_Test_CMA/export/wall_cart.geo");
        let mut reader = ensight_gold::Reader::new(path).unwrap();

        let geo = ensight_gold::Geometry::read(&mut reader);
    }
}