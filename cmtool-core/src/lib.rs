use std::{path::Path, sync::Arc};

use cmtool_data::RawData;

use crate::model::{scalar::Scalar, CMGeometry, CMModel};

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

use thiserror::Error;
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Cmtool: {0}")]
    Data(#[from] cmtool_data::DataError),

    #[error("Cmtool: {0}")]
    Custom(String),

    #[error("Handle")]
    Handle,

    #[error("Error writing/reading file: {0}")]
    IO(#[from] std::io::Error),
}

pub struct CMHandle {
    model: model::CMModel,
    root_result: String,
    eg_geometry: Arc<ensight_gold::Geometry>,
    cm_geometry: Arc<CMGeometry>,
}

impl CMHandle {
    pub fn init(
        n_div: [usize; 3],
        root: &str,
        geometry_filename: &str,
        _meshtype: grid::MeshType,
    ) -> Result<Self, CoreError> {
        let fullpath = format!("{}/{}", root, geometry_filename);

        let task_io =
            std::thread::spawn(move || ensight_gold::Geometry::new(Path::new(&fullpath.clone())));

        let eg_geometry = Arc::new(
            task_io
                .join()
                .map_err(|_| CoreError::Custom("Thread error".to_string()))?
                .map_err(|_| CoreError::Custom("Arc error".to_string()))?,
        ); //FIXME

        println!("{:?}", eg_geometry);

        let cm_geometry = Arc::new(CMGeometry::init(n_div, eg_geometry.clone(), grid::MeshType::Cylindrical));

        // let fullpath = format!("{}/wall_cart.scl1", root);
        // let s = ensight_gold::scalar::ScalarField::init(eg_geometry, Path::new(&fullpath.clone()))?;

        Ok(Self {
            model: CMModel::init(cm_geometry.clone()),
            root_result: String::from("./test"),
            eg_geometry,
            cm_geometry
        })
    }

    pub fn dump_volume(&self) {
        self.model.compartments_volumes();

        todo!()
    }
    pub fn dump_scalar(&self, path: impl AsRef<std::path::Path>) -> Result<(), CoreError> {

        let s = ensight_gold::scalar::ScalarField::init(self.eg_geometry.clone(), path)?;

        let scalar = Scalar::new(s,&self.cm_geometry,&self.eg_geometry);
        
        let scalar_data = self.model.export_volume_integral_per_zone(scalar)?;

        scalar_data.write_raw("test.scraw")?;

        Ok(())
    }

    pub fn dump_vector(&self) -> Result<(), CoreError> {
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
