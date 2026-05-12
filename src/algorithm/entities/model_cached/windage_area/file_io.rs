
use std::path::PathBuf;
use std::fs::File;
use sal_core::dbg::Dbg;
use sal_core::error::Error;

//
pub fn read(dbg: &Dbg, cache_path: &PathBuf) -> Result<Vec<f64>, Error> {
    let error = Error::new(dbg, "read");
    let mut file = File::open(cache_path).map_err(|err| error.pass_with("File::open", err.to_string()))?;
    let data: Vec<f64> = match bincode::decode_from_std_read(&mut file, bincode::config::standard()) {
        Ok(data) => Ok(data),
        Err(err) => Err(error.pass_with("Encode error", err.to_string())),
    }.map_err(|err| error.pass_with("decode_from_std_read", err.to_string()))?;
    Ok(data)
}
//
pub fn save(dbg: &Dbg, cache_path: &PathBuf, data: &Vec<f64>) -> Result<(), Error> {
    let error = Error::new(dbg, "save");
    let mut file = File::create(cache_path).map_err(|err| error.pass_with("File::create", err.to_string()))?;
    bincode::encode_into_std_write(data, &mut file, bincode::config::standard())
        .map_err(|err| error.pass_with("bincode::encode_into_writer", err.to_string()))?;
    Ok(())
}
