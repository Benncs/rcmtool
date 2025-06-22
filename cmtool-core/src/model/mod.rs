use std::{io::Write, fs::File, io::BufWriter};

use crate::model::scalar::Scalar;
pub mod scalar;
struct CMGeomtry {

    n_zones:usize
}

impl CMGeomtry
{
    pub fn init(n_div:[usize;3])->Self
    {

        Self{n_zones:n_div.iter().product::<usize>()}
    }
}


pub struct CMModel {
    geometry: CMGeomtry,
}

impl  CMModel
{

}

impl CMModel {
    fn init(geometry: CMGeomtry) -> Self {
        Self { geometry }
    }

    fn compute_flux_through_limits() -> Vec<f64> {
        todo!()
    }

    fn compute_volume_integral_per_zone() -> Vec<f64> {
        todo!()
    }

    pub fn export_flux_through_limits(&self,flow:&mut cmtool_data::RawDataFlux) {
        todo!()
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    pub fn export_volume_integral_per_zone(&self,scalar:Scalar) ->Result<cmtool_data::RawDataScalar,()>{
        todo!()
    }

    pub fn compartments_volumes(&self)->Vec<f64>
    {
        todo!()
    }

    pub fn get_real_volume(&self)->&[f64]
    {
        todo!()
    }
}
