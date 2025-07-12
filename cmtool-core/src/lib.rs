use std::{
    mem::uninitialized,
    path::{Path, PathBuf},
    sync::Arc,
};

use cmtool_data::RawData;

use crate::{
    ensight_gold::{RawField, VariableInfo},
    model::{CMGeometry, CMModel, Scalar, Vector},
};

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
    model: Arc<model::CMModel>,
    root_result: String,
    eg_geometry: Arc<ensight_gold::Geometry>,
    cm_geometry: Arc<CMGeometry>,
}

fn resolve_path(
    root: &impl AsRef<std::path::Path>,
    relative_path: &str,
) -> impl AsRef<std::path::Path> {
    std::path::PathBuf::from(root.as_ref()).join(relative_path)
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

        println!("{}", eg_geometry);

        let cm_geometry = Arc::new(CMGeometry::init(
            n_div,
            eg_geometry.clone(),
            grid::MeshType::Cylindrical,
        ));

        // let fullpath = format!("{}/wall_cart.scl1", root);
        // let s = ensight_gold::scalar::ScalarField::init(eg_geometry, Path::new(&fullpath.clone()))?;

        Ok(Self {
            model: Arc::new(CMModel::init(cm_geometry.clone())),
            root_result: String::from("./test"),
            eg_geometry,
            cm_geometry,
        })
    }

    pub fn dump_volume(&self) {
        self.model.compartments_volumes();

        todo!()
    }

    pub fn dump_all(
        &self,
        root_export: impl AsRef<std::path::Path>,
        root_input: impl AsRef<std::path::Path>,
        vars: &[ensight_gold::VariableInfo],
    ) -> Result<(), CoreError> {
        std::fs::create_dir(&root_export)?;

        // let mut handles = Vec::new();
        for v in vars.iter() {
            // let res_name = resolve_path(&root_export, &v.name)
            //     .as_ref()
            //     .to_str()
            //     .unwrap()
            //     .to_owned();

            // let path = resolve_path(&root_input, &v.filepath)
            //     .as_ref()
            //     .to_str()
            //     .unwrap()
            //     .to_owned();
            // let eg_geometry_clone = self.eg_geometry.clone();
            // let cm_geometry_clone = self.cm_geometry.clone();
            // let model_clone = self.model.clone();
            match v.get_type() {
                ensight_gold::VariableType::Scalar => {
                    self.dump_scalar(
                        resolve_path(&root_export, &v.name),
                        resolve_path(&root_input, &v.filepath),
                    )?;

                    // let eg_geometry_clone = self.eg_geometry.clone();
                    // let cm_geometry_clone = self.cm_geometry.clone();
                    // let model_clone = self.model.clone();

                    // handles.push(std::thread::spawn(move || {
                    //     Self::ts_dump_scalar(
                    //         res_name,
                    //         path,
                    //         eg_geometry_clone,
                    //         cm_geometry_clone,
                    //         model_clone,
                    //     )
                    // }));
                }
                ensight_gold::VariableType::Vector => {
                    self.dump_vector(
                        resolve_path(&root_export, &v.name),
                        resolve_path(&root_input, &v.filepath),
                    )?;
                }
            }
        }

        // for i in handles
        // {
        //     match i.join() {
        //                 Ok(Ok(_)) => println!("Thread executed successfully"),
        //                 Ok(Err(e)) => println!("Thread failed with error: {:?}", e),
        //                 Err(e) => println!("Thread panicked: {:?}", e),
        //             }
        // }

        Ok(())
    }

    fn ts_dump_scalar(
        res_name: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
        eg_geometry: Arc<ensight_gold::Geometry>,
        cm_geometry: Arc<CMGeometry>,
        model: Arc<model::CMModel>,
    ) -> Result<(), CoreError> {
        let s = ensight_gold::scalar::ScalarField::init(eg_geometry.clone(), path)?;

        let scalar = Scalar::new(s, &cm_geometry, &eg_geometry);

        let scalar_data = model.export_volume_integral_per_zone(scalar)?;

        scalar_data.write_raw(&format!("{}.raw", res_name.as_ref().to_str().unwrap()))?;

        Ok(())
    }

    pub fn dump_scalar(
        &self,
        res_name: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), CoreError> {
        Self::ts_dump_scalar(
            res_name,
            path,
            self.eg_geometry.clone(),
            self.cm_geometry.clone(),
            self.model.clone(),
        )
    }

    pub fn dump_vector(
        &self,
        res_name: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), CoreError> {
        let n_zone = 10;
        let n_flux = 20;

        let v = ensight_gold::vectors::VectorField::init(self.eg_geometry.clone(), path)?;
        let vector = Vector::new(v, &self.cm_geometry, &self.eg_geometry);
        todo!("dump vector");
        let flow_data = self.model.export_flux_through_limits(vector)?;

        todo!()
    }

    pub fn dump_vector_from_scalar(&self) {
        todo!()
    }

    pub fn export_geometry_compartments(&self) {
        todo!()
    }
}
