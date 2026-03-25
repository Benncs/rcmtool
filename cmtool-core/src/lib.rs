// SPDX-License-Identifier: GPL-3.0-or-later

pub mod coordinates;
pub mod ensight_gold;
mod errors;
pub mod grid;
pub mod model;
pub mod utils;
pub use errors::CoreError;

#[cfg(feature = "use_vtk")]
use grid::vtk::VtkCm;
#[cfg(feature = "use_vtk")]
use grid::vtk::add_celldata_to_vtk;

use cmtool_data::{RawData, RawDataFlux, RawDataScalar};
use model::{CMGeometry, CMModel, Scalar, Vector};
use std::cmp::Ordering;
use std::{path::Path, sync::Arc};

fn resolve_path(
    root: &impl AsRef<std::path::Path>,
    relative_path: &str,
) -> impl AsRef<std::path::Path> {
    std::path::PathBuf::from(root.as_ref()).join(relative_path)
}

pub enum ExportType {
    EnsightGold,
}

// trait GeometryInfo {}

// trait CfdCase {
//     fn get_root(&self) -> String;
//     fn get_geometry_relative_path(&self) -> String;
// }

pub struct CMHandle {
    model: Arc<model::CMModel>,
    _root_result: String, //TODO EITHER USE IT OR REMOVE
    eg_geometry: Arc<ensight_gold::Geometry>,
    cm_geometry: Arc<CMGeometry>,
}

impl CMHandle {
    pub fn grid(&self) -> &dyn crate::grid::CompartmentMesh {
        self.model.grid()
    }

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

        Ok(Self {
            model: Arc::new(CMModel::init(cm_geometry.clone())),
            _root_result: String::from("./test"),
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
        vars: &[ensight_gold::case::VariableInfo],
    ) -> Result<(), CoreError> {
        std::fs::create_dir_all(&root_export)?;

        let mut rs = Vec::with_capacity(vars.len());
        let mut v: Vec<ensight_gold::case::VariableInfo> = vars.to_owned();

        v.sort_by(|a, b| {
            if a.get_type() == ensight_gold::case::VariableType::Scalar {
                Ordering::Less
            }
            // } else if b.get_type() == ensight_gold::case::VariableType::Scalar {
            //     Ordering::Less
            // }
            else {
                Ordering::Equal
            }
        });

        for v in vars.iter() {
            match v.get_type() {
                ensight_gold::case::VariableType::Scalar => {
                    let sc = self.dump_scalar(
                        resolve_path(&root_export, &v.name),
                        resolve_path(&root_input, &v.filepath),
                    )?;
                    rs.push((sc, v.name.clone()));
                }
                ensight_gold::case::VariableType::Vector => {
                    self.dump_vector(
                        resolve_path(&root_export, &v.name),
                        resolve_path(&root_input, &v.filepath),
                    )?;
                }
            }
        }

        #[cfg(feature = "use_vtk")]
        self.export_vtk(
            format!("{}/cma_case.vtu", root_export.as_ref().display()),
            rs,
        )?;

        Ok(())
    }

    pub fn get_scalar(&self, path: impl AsRef<std::path::Path>) -> Result<Scalar, CoreError> {
        Self::s_get_scalar(path, self.eg_geometry.clone(), self.cm_geometry.clone())
    }

    fn s_get_scalar(
        path: impl AsRef<std::path::Path>,
        eg_geometry: Arc<ensight_gold::Geometry>,
        cm_geometry: Arc<CMGeometry>,
    ) -> Result<Scalar, CoreError> {
        let s = ensight_gold::scalar::ScalarField::init(eg_geometry.clone(), path)?;
        Ok(Scalar::new(s, &cm_geometry, &eg_geometry))
    }

    fn ts_dump_scalar(
        res_name: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
        eg_geometry: Arc<ensight_gold::Geometry>,
        cm_geometry: Arc<CMGeometry>,
        model: Arc<model::CMModel>,
    ) -> Result<RawDataScalar, CoreError> {
        let scalar = Self::s_get_scalar(path, eg_geometry, cm_geometry)?;

        let scalar_data = model.export_volume_integral_per_zone(scalar)?;

        scalar_data.write_raw(&format!("{}.raw", res_name.as_ref().to_str().unwrap()))?;

        Ok(scalar_data)
    }

    pub fn dump_scalar(
        &self,
        res_name: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<RawDataScalar, CoreError> {
        Self::ts_dump_scalar(
            res_name,
            path,
            self.eg_geometry.clone(),
            self.cm_geometry.clone(),
            self.model.clone(),
        )
    }

    pub fn dump_real_volume(&self, res_name: impl AsRef<std::path::Path>) -> Result<(), CoreError> {
        let volumes_data: RawDataScalar = self.model.get_real_volume().into();

        volumes_data.write_raw(&format!("{}.raw", res_name.as_ref().to_str().unwrap()))?;

        Ok(())
    }

    pub fn dump_vector_phase_fraction(
        &self,
        res_name: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
        phase_fraction: Scalar,
    ) -> Result<(), CoreError> {
        let v = ensight_gold::vectors::VectorField::init(self.eg_geometry.clone(), path)?;
        let vector =
            Vector::new(v, &self.cm_geometry, &self.eg_geometry).scale_by(phase_fraction)?;

        let flow_data = self.model.compute_flux_between_compartments(vector)?;

        flow_data.write_raw(&format!("{}.raw", res_name.as_ref().to_str().unwrap()))?;
        Ok(())
    }

    pub fn dump_vector_raw(
        &self,
        res_name: impl AsRef<std::path::Path>,
        vector: Vector,
    ) -> Result<(), CoreError> {
        let flow_data = self.model.compute_flux_between_compartments(vector)?;

        flow_data.write_raw(&format!("{}.raw", res_name.as_ref().to_str().unwrap()))?;
        Ok(())
    }

    pub fn dump_vector(
        &self,
        res_name: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), CoreError> {
        let v = ensight_gold::vectors::VectorField::init(self.eg_geometry.clone(), path)?;
        let vector = Vector::new(v, &self.cm_geometry, &self.eg_geometry);
        let flow_data = self.model.compute_flux_between_compartments(vector)?;

        flow_data.write_raw(&format!("{}.raw", res_name.as_ref().to_str().unwrap()))?;
        Ok(())
    }

    pub fn vector_from_scalar(
        &self,
        path_i: impl AsRef<std::path::Path>,
        path_j: impl AsRef<std::path::Path>,
        path_k: impl AsRef<std::path::Path>,
    ) -> Result<Vector, CoreError> {
        let s = ensight_gold::scalar::ScalarField::init(self.eg_geometry.clone(), path_i)?;
        let sj = ensight_gold::scalar::ScalarField::init(self.eg_geometry.clone(), path_j)?;
        let sk = ensight_gold::scalar::ScalarField::init(self.eg_geometry.clone(), path_k)?;
        Vector::from_scalar([s, sj, sk], &self.cm_geometry, &self.eg_geometry)
    }

    pub fn dump_vector_from_scalar(
        &self,
        res_name: impl AsRef<std::path::Path>,
        path_i: impl AsRef<std::path::Path>,
        path_j: impl AsRef<std::path::Path>,
        path_k: impl AsRef<std::path::Path>,
    ) -> Result<RawDataFlux, CoreError> {
        let vector = self.vector_from_scalar(path_i, path_j, path_k)?;
        let flow_data = self.model.compute_flux_between_compartments(vector)?;

        flow_data.write_raw(&format!("{}.raw", res_name.as_ref().to_str().unwrap()))?;
        Ok(flow_data)
    }

    pub fn export_geometry_compartments(&self) {
        todo!()
    }

    // #[cfg(feature = "use_vtk")]
    // pub fn write_vtk(&self, path: impl AsRef<std::path::Path>) -> Result<(), CoreError> {
    //     let mesh = self.cm_geometry.get_grid().unwrap();
    //     let p = path.as_ref().to_str().unwrap();
    //     let mut vtk = mesh.get_vtk(p)?;

    //     let volumes_data = self.model.get_real_volume();

    //     let volumes_data_array = vtkio::model::DataArray::scalars("real_volume", 1);

    //     let volumes_data_array = volumes_data_array.with_vec(volumes_data);

    //     add_celldata_to_vtk(
    //         &mut vtk,
    //         vtkio::model::Attribute::DataArray(volumes_data_array),
    //     );

    //     let mut vtk_bytes = Vec::<u8>::new();
    //     vtk.write_xml(&mut vtk_bytes).unwrap();
    //     std::fs::write(path, vtk_bytes).unwrap();

    //     Ok(())
    // }

    #[cfg(feature = "use_vtk")]
    fn export_vtk(
        &self,
        path: impl AsRef<std::path::Path>,
        sc: Vec<(RawDataScalar, String)>,
    ) -> Result<(), CoreError> {
        let mesh = self.cm_geometry.get_grid().unwrap();
        let p = path.as_ref().to_str().unwrap();
        let mut vtk = mesh.get_vtk(p)?;

        let mut add_cell = |name: &str, data: Vec<f64>| {
            let data_array = vtkio::model::DataArray::scalars(name, 1);
            let data_array = data_array.with_vec(data);
            add_celldata_to_vtk(&mut vtk, vtkio::model::Attribute::DataArray(data_array));
        };

        sc.iter().for_each(|(r, n)| {
            let ve: Vec<f64> = r.values.iter().map(|i| i.value).collect();
            add_cell(n, ve)
        });

        let volumes_data = self.model.get_real_volume();
        add_cell("real_volume", volumes_data);

        let mut vtk_bytes = Vec::<u8>::new();
        vtk.write_xml(&mut vtk_bytes).unwrap();
        std::fs::write(path, vtk_bytes).unwrap();

        Ok(())
    }
}
