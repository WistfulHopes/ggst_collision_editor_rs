use std::{path::PathBuf, fs::File, io::{Read}};
use arcsys::{ggst::pac::{GGSTPac}, Error};

pub fn open_file(path: &PathBuf) -> Result<GGSTPac, arcsys::Error> {
    let mut file_buf = Vec::new();
    if let Err(e) = File::open(&path).and_then(|mut f| f.read_to_end(&mut file_buf)) {
        println!("Error reading file {}: {}", path.display(), e);
        return Err(Error::Parser("couldn't open file".to_string()));
    };
    match GGSTPac::parse(&file_buf)
    {
        Ok(file) => return Ok(file),
        Err(e) => return Err(e),
    };
}