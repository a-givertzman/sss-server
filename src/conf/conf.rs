use std::{fs::OpenOptions, path::Path};

use serde::{Deserialize, Serialize};

use super::api_conf::ApiConf;

#[derive(Serialize, Deserialize)]
pub struct Conf {
    pub api: ApiConf,
}
//
//
impl Conf {
    pub fn new<P: AsRef<Path>>(parent: impl Into<String>, path: P) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .unwrap();
        let api = serde_yaml::from_reader(file).unwrap();
        Self {
            api, 
        }
    }
}
