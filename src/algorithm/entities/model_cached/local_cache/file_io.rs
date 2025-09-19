use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use sal_core::{dbg::Dbg, error::Error};

///
/// read cache data from `path` file.
///
/// # Panics
/// Panic occurs if the reader produces a non-comparable value (e. g. _NaN_).
pub fn read(dbg: &Dbg, cache_path: &PathBuf) -> Result<Vec<Vec<f64>>, Error> {
    let error = Error::new(dbg, "read");
    let parent_dir = cache_path.parent().ok_or(error.err(format!(
        "cache_path.parent error! path:{}",
        cache_path.display()
    )))?;
    std::fs::create_dir_all(parent_dir).map_err(|err| {
        error.pass_with(
            format!("std::fs::create_dir_all error! path:{}", cache_path.display()),
            err.to_string(),
        )
    })?;
    let callee = "read_from_file";
    let file = File::open(cache_path).map_err(|err| {
        format!(
            "{}.{} | Failed reading file='{}': {}",
            dbg,
            callee,
            cache_path.display(),
            err
        )
    })?;
    let reader = BufReader::new(file);
    let mut vals = Vec::new();
    for (try_line, line_id) in reader.lines().zip(1..) {
        let mut v = Vec::new();
        let line = try_line.map_err(|err| {
            format!(
                "{}.{} | Failed reading line={}: {}",
                dbg, callee, line_id, err
            )
        })?;
        let ss = line.split_ascii_whitespace();
        for s in ss {
            let val = s.parse().map_err(|err| {
                format!(
                    "{}.{} | Failed parsing value at line={}: {}",
                    dbg, callee, line_id, err
                )
            })?;
            v.push(val);
        }
        vals.push(v);
    }
    let size = vals
        .first()
        .ok_or(format!("{}.{} | Error: no vals", dbg, callee,))?
        .len();
    for v in &vals {
        if v.len() != size {
            return Err(format!("{}.{} | Error: no vals", dbg, callee,).into());
        }
    }
    Ok(vals)
}
///
pub fn save(dbg: &Dbg, cache_path: &PathBuf, vals: Vec<Vec<f64>>) -> Result<(), Error> {
    let error = Error::new(dbg, "save");
    let parent_dir = cache_path.parent().ok_or(error.err(format!(
        "cache_path.parent error! path:{}",
        cache_path.display()
    )))?;
    std::fs::create_dir_all(parent_dir).map_err(|err| {
        error.pass_with(
            format!("std::fs::create_dir_all error! path:{}", cache_path.display()),
            err.to_string(),
        )
    })?;
    let mut file = File::create(cache_path).map_err(|err| {
        error.pass_with(
            format!("File::create error! path:{}", cache_path.display()),
            err.to_string(),
        )
    })?;
    for col in vals.iter() {
        let cols_str: Vec<_> = col.iter().map(ToString::to_string).collect();
        let line = cols_str.join("\t");
        writeln!(&mut file, "{}", line).map_err(|err| {
            error.pass_with(
                format!("Writing to file, path:{}", cache_path.display()),
                err.to_string(),
            )
        })?;
    }
    Ok(())
}
