use crate::kernel::dbgid::dbgid::DbgId;
use crate::kernel::str_err::str_err::StrErr;
use rustc_hash::FxHashMap;
use serde::Serialize;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::{Path, PathBuf};
///
/// Storage data from json files
pub struct Storage {
    dbgid: DbgId,
    /// cash-storage
    hash: FxHashMap<String, Value>,
    /// path to file json
    path: PathBuf,
}
//
//
impl Storage {
    ///
    /// New instance [Storage]
    /// - `file_path` - path to file json
    pub fn new(path: impl AsRef<Path>) -> Self {
        Storage {
            dbgid: DbgId("Storage".to_string()),
            hash: FxHashMap::default(),
            path: path.as_ref().to_path_buf(),
        }
    }
    ///
    /// Method to load data from file json
    /// - 'key' - key to data
    pub fn load(&mut self, key: &str) -> Result<Value, StrErr> {
        let key = key.trim_start_matches('.').to_owned();
        if key.is_empty() {
            return Err(StrErr(format!("{}.load | Key can't be empty", self.dbgid)));
        } else if key.contains("..") || key.contains('\\') {
            return Err(StrErr(format!(
                "{}.load | Invalid structure of key",
                self.dbgid
            )));
        }
        match self.hash.get(&key) {
            Some(value) => Ok(value.clone()),
            None => {
                let path = self.path.join(&key);
                let file = OpenOptions::new().read(true).open(&path).map_err(|err| {
                    StrErr(format!(
                        "{}.load | Failed to open file: {:?}, error: {}",
                        self.dbgid, path, err
                    ))
                })?;
                match serde_json::from_reader::<_, Value>(BufReader::new(file)) {
                    Ok(json_value) => {
                        self.hash.insert(key, json_value.clone());
                        Ok(json_value)
                    }
                    Err(err) => Err(StrErr(format!(
                        "{}.load | Parse error: {} in the file: {:?}",
                        self.dbgid, err, path
                    ))),
                }
            }
        }
    }
    ///
    /// Method to store data by key
    /// - `key` - key to data
    /// - `value` - value to store
    pub fn store(&mut self, key: &str, value: impl Serialize) -> Result<(), StrErr> {
        let key = key.trim_start_matches('.').to_owned();
        if key.is_empty() {
            return Err(StrErr(format!("{}.store | Key can't be empty", self.dbgid)));
        } else if key.contains("..") || key.contains('\\') {
            return Err(StrErr(format!(
                "{}.store | Invalid structure of key",
                self.dbgid
            )));
        }
        let path = self.path.join(&key);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|err| {
                StrErr(format!(
                    "{}.store | Error on path '{}': {:?}",
                    self.dbgid,
                    path.display(),
                    err
                ))
            })?;
        match serde_json::to_writer_pretty(file, &value) {
            Ok(_) => Ok(()),
            Err(err) => Err(StrErr(format!(
                "{}.store | Parse error on path '{}': {}",
                self.dbgid,
                path.display(),
                err
            ))),
        }
    }
}
