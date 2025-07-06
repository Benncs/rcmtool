use std::{collections::BTreeSet, sync::Arc};

use crate::{
    ensight_gold::{
        self,
        types::{ElementsType, VolumeElementTypes},
        Part,
    },
    grid::{
        cylindrical_index, get_mesh, AxisDescriptor, CompartmentMesh, Coords3, CylindricalAxis,
        MeshType,
    },
    model::scalar::Scalar,
    CoreError,
};
mod data;
pub mod scalar;
use data::*;
mod geometry;
use geometry::*;
pub struct CMModel {
    geometry: Arc<CMGeometry>,
}

pub use geometry::CMGeometry;

impl CMModel {}

impl CMModel {
    pub fn init(geometry: Arc<CMGeometry>) -> Self {
        Self { geometry }
    }

    fn compute_flux_through_limits() -> Vec<f64> {
        todo!()
    }

    fn compute_volume_integral_per_zone() -> Vec<f64> {
        todo!()
    }

    pub fn export_flux_through_limits(&self, flow: &mut cmtool_data::RawDataFlux) {
        todo!()
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    pub fn export_volume_integral_per_zone(
        &self,
        scalar: Scalar,
    ) -> Result<cmtool_data::RawDataScalar, CoreError> {
        todo!()
    }

    pub fn compartments_volumes(&self) -> Vec<f64> {
        todo!()
    }

    pub fn get_real_volume(&self) -> &[f64] {
        todo!()
    }
}
