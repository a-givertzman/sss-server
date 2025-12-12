
use std::path::PathBuf;
use std::fs::File;
use bincode::{Decode, Encode};
use sal_core::dbg::Dbg;
use sal_core::error::Error;

use crate::algorithm::entities::model_cached::AreaData;
///
pub fn read(dbg: &Dbg, cache_path: &PathBuf) -> Result<AreaData, Error> {
    let error = Error::new(dbg, "read");
    let mut file = File::open(cache_path).map_err(|err| error.pass_with("File::open", err.to_string()))?;
    let mut data: Vec<(f64, Vec<(f64, f64)>)> = match bincode::decode_from_std_read(&mut file, bincode::config::standard()) {
        Ok(data) => Ok(data),
        Err(err) => Err(error.pass_with("Encode error", err.to_string())),
    }.map_err(|err| error.pass_with("decode_from_std_read", err.to_string()))?;
    Ok(AreaData {
        x_end: data.pop().ok_or(error.err("no x_end"))?.0, 
        x_start: data.pop().ok_or(error.err("no x_start"))?.0, 
        voxels: data,
    })
}
///
pub fn save(dbg: &Dbg, cache_path: &PathBuf, mut data: AreaData) -> Result<(), Error> {
    let error = Error::new(dbg, "save");
    let mut file = File::create(cache_path).map_err(|err| error.pass_with("File::create", err.to_string()))?;
    let mut save_data = Vec::new();
    save_data.append(&mut data.voxels);
    save_data.push((data.x_start, vec![]));
    save_data.push((data.x_end, vec![]));
    bincode::encode_into_std_write(save_data, &mut file, bincode::config::standard())
        .map_err(|err| error.pass_with("bincode::encode_into_writer", err.to_string()))?;
    Ok(())
}
