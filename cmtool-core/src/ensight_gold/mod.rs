mod reader;
use std::{
    fmt::Debug, fs::{read, File}, io::{BufReader, Error, ErrorKind}, path::Path, str::FromStr
};
mod geo;
mod types;
use crate::ensight_gold::types::ElementsType;
pub use crate::{ensight_gold::reader::Reader, utils};
pub mod scalar;


pub use geo::Geometry;

struct Case{
    geometry_file_path:String
}

impl Case
{
    pub fn read(path:&Path)->std::io::Result<()>
    {
        let fd = File::open(path)?;
        let buffer=  BufReader::new(fd);


        

        todo!()
            
        
    }
}




