use std::{fs::OpenOptions, path::Path};
use sal_core::dbg::Dbg;
use serde::Deserialize;
use crate::{conf::CalculusConf, server::ServerConf};
use super::{api_conf::ApiConf, thread_pool_conf::ThreadPoolConf};

#[derive(Debug, Clone, Deserialize)]
pub struct Conf {
    pub api: ApiConf,
    #[serde(alias="thread-pool")]
    pub thread_pool: ThreadPoolConf,
    pub server: ServerConf,
    pub algorithm: CalculusConf,
}
//
//
impl Conf {
    pub fn new<P: AsRef<Path>>(parent: impl Into<String>, path: P) -> Self {
        let dbg = Dbg::new(parent, "Conf");
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .unwrap();
        match serde_yaml::from_reader(file) {
            Ok(conf) => {
                let conf: Conf = conf;
                conf
            }
            Err(err) => panic!("{dbg}, Error: {:?}", err),
        }
    }
}
