use std::{path::Path, sync::Arc};

use cmtool_data::RawData;

use crate::{ensight_gold::Reader, model::scalar::Scalar};

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
    root_result: String,
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

        println!("{:?}", eg_geometry);

        // let fullpath = format!("{}/wall_cart.scl1", root);
        // let s = ensight_gold::scalar::ScalarField::init(eg_geometry, Path::new(&fullpath.clone()))
        //     .unwrap();
        // println!("{:?}", s);

        todo!()
    }

    pub fn dump_volume(&self) {
        self.model.compartments_volumes();

        todo!()
    }
    pub fn dump_scalar(&self) -> Result<(), ()> {
        let n_zone = 10;

        let scalar = Scalar::new();
        let scalar_data = self.model.export_volume_integral_per_zone(scalar)?;

        
        let path = todo!();
        scalar_data.write_raw(path)
    }

    pub fn dump_vector(&self) {
        let n_zone = 10;
        let n_flux = 20;
        let mut flow_data = cmtool_data::RawDataFlux::new(n_zone, n_flux);

        self.model.export_flux_through_limits(&mut flow_data);

        todo!()
    }

    pub fn dump_vector_from_scalar(&self) {
        todo!()
    }

    pub fn export_geometry_compartments(&self) {
        todo!()
    }
}
