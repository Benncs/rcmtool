use std::io::Write;

use serde::{Deserialize, Serialize};

#[repr(C, packed)]
#[derive(Deserialize, Serialize)]
pub struct FluxFileHeader {
    pub n_zone: u32,
    pub n_max: u32,
}



#[repr(C, packed)]
#[derive(Deserialize, Serialize)]
pub struct ScalarFileHeader {
    pub n_zone: u32,
}

#[repr(C, packed)]
#[derive(Deserialize, Serialize)]
pub struct RawFlux {
    pub id_source: u32,
    pub id_target: u32,
    pub flux_source_target: f64,
    pub flux_target_source: f64,
}

#[repr(C, packed)]
#[derive(Deserialize, Serialize)]
pub struct RawScalar {
    pub value: f64,
}

#[derive(Deserialize, Serialize)]
pub struct RawDataScalar {
    pub header: ScalarFileHeader,
    pub values: Vec<RawScalar>,
}
#[derive(Deserialize, Serialize)]
pub struct RawDataFlux {
    pub header: FluxFileHeader,
    pub fluxes: Vec<RawFlux>,
}

pub trait RawData:where Self: std::marker::Sized
{
    fn read(path: &str) -> Option<Self> ;
    fn write(&self, path: &str);
}




// impl RawData for RawDataScalar
// {
//     fn read(path: &str) -> Option<Self> {
        
//     }
//     fn write(&self, path: &str) {
//         let data = self.serialize(serializer);
//         let mut file = File::create(path)?;
//         file.write_all(xml.as_bytes())?;
//         Ok(())
//     }
// }