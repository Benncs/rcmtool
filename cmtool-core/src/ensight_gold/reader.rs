use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::path::Display;
use std::path::Path;


pub struct FileBuffer<const N:usize>([u8; N]);



impl<const N:usize> FileBuffer<N> {
    pub fn to_string(&self) -> String {
        let trimmed = match self.0.iter().position(|&b| b == 0) {
            Some(pos) => &self.0[..pos],
            None => &self.0[..],
        };

        String::from_utf8_lossy(trimmed).into_owned()
    }
}

pub struct Reader<const N:usize> {
    reader: BufReader<File>,
    line_buffer: FileBuffer<N>,
}

impl<const N:usize> Reader<N> {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let fd = File::open(path)?;
        Ok(Reader {
            reader: BufReader::new(fd),
            line_buffer: FileBuffer([0; N]),
        })
    }

    pub fn check_lines_contains(&mut self,name:&str)->std::io::Result<()>
    {
        if !self.get_line()?.to_string().contains(name) {
            return Err(std::io::Error::new(
                ErrorKind::Unsupported,
                format!("Missing '{}' in header",name),
            ));
        }
        Ok(())
    }
    pub fn checK_eof(&mut self)->std::io::Result<bool>
    {
        let buffer = self.reader.fill_buf()?; //Fill_buff does not consum, keep current file position
        if buffer.is_empty() {
            Ok(true)  // EOF
        } else {
            
            Ok(false) // Not EOF
        }
    }

    fn get_line(&mut self) -> std::io::Result<&FileBuffer<N>> {
        self.reader.read_exact(&mut self.line_buffer.0)?;
        Ok(&self.line_buffer)
    }

    pub fn get_line_string(&mut self)->std::io::Result<String>
    {
        self.reader.read_exact(&mut self.line_buffer.0)?;
        Ok(self.line_buffer.to_string())
    }

    pub fn ignore_line(&mut self) -> std::io::Result<()> {
        self.ignore_bytes(N as i64)
    }

    pub fn ignore_bytes(&mut self, n: i64) -> std::io::Result<()> {
        self.reader.seek(SeekFrom::Current(n))?;
        Ok(())
    }

    pub fn rollback_n(&mut self, n: i64) -> std::io::Result<()> {
        if n < 0 {
            return Err(std::io::Error::new(
                ErrorKind::Unsupported,
                "Rollback parameter must be positive",
            ));
        }
        self.reader.seek(SeekFrom::Current(-n))?;
        Ok(())
    }

    pub fn rollback(&mut self) -> std::io::Result<()> {
        self.rollback_n(80)
    }

    pub fn read_i32(&mut self) -> std::io::Result<i32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }
    pub fn read_f32(&mut self) -> std::io::Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }
}
const ENSIGHT_GOLDER_BINARY_FORMAT_LINE_SIZE : usize = 80; //Bytes; 
pub type EnsightGoldReader = Reader<ENSIGHT_GOLDER_BINARY_FORMAT_LINE_SIZE>;