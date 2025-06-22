use std::{path::Path, sync::Arc};

use crate::ensight_gold::Reader;

pub mod ensight_gold;
pub mod grid;
pub mod model;
pub mod utils;
pub enum ExportType {
    EnsightGold,
}

trait GeometryInfo {}

trait CfdCase {
    fn get_root(&self) -> String;
    fn get_geometry_relative_path(&self) -> String;
}

pub struct CMHandle {
    model: model::CMModel,
}

impl CMHandle {
    pub fn init(
        _n_div: [usize; 3],
        root: &str,
        geometry_filename: &str,
        _meshtype: grid::MeshType,
    ) -> Result<Self, ()> {
        let fullpath = format!("{}/{}", root, geometry_filename);

        let task_io =
            std::thread::spawn(move || ensight_gold::Geometry::new(Path::new(&fullpath.clone())));

        let eg_geometry = Arc::new(task_io.join().map_err(|_| ())?.map_err(|_| ())?); //FIXME
        
        println!("{:?}",eg_geometry);

        let fullpath = format!("{}/wall_cart.scl1", root);
        

        let s = ensight_gold::scalar::ScalarField::init(eg_geometry, Path::new(&fullpath.clone())).unwrap();
        
        println!("{:?}",s);

        todo!()
    }

    pub fn dump_volume(&self) {
        self.model.compartments_volumes();

        todo!()
    }
    pub fn dump_scalar(&self) {
        let mut buffer: Vec<u8> = Vec::new();

        self.model.export_volume_integral_per_zone(&mut buffer);

        todo!()
    }

    pub fn dump_vector(&self) {
        let mut buffer: Vec<u8> = Vec::new();

        self.model.export_flux_through_limits(&mut buffer);

        todo!()
    }

    pub fn dump_vector_from_scalar(&self)
    {
        todo!()
    }

    pub fn export_geometry_compartments(&self)
    {
        todo!()
    }
}
