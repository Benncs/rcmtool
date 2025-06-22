use std::{io::Write, fs::File, io::BufWriter};

struct CMGeomtry {}

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

    pub fn export_flux_through_limits<W: Write>(&self,writer: &mut W) {
        todo!()
    }

    pub fn export_volume_integral_per_zone<W: Write>(&self,writer: &mut W) {
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
