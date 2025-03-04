use std::path::{Path, PathBuf};

use crate::kernel::run::Run;

pub struct App {
    path: PathBuf,
}
//
//
impl App {
    ///
    /// Returns new instance of App
    /// - path - relative path to the config (yaml)
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().into(),
        }
    }
}
//
//
impl Run for App {
    //
    //
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
}
