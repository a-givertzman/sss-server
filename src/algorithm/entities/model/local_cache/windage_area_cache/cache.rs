use crate::{
    algorithm::entities::{
        cache::Cache,
        model::{Shape, local_cache::LocalCache},
    },
    kernel::types::RwLock,
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::Scheduler;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
///
/// Pre-calculated cache for floating position algorithm.
pub struct AreaCache {
    dbg: Dbg,
    cache_path: PathBuf,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    ///
    /// Model representation used for cache calculation.
    shape: Shape,
    ///
    /// Cache read from `self.file_path`.
    cache: Arc<RwLock<Option<Cache<f64>>>>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl AreaCache {
    //
    //
    const KEY: &'static str = "floating_position_cache";
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        shape: Shape,
        cache_dir: impl AsRef<Path>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, "AreaCache");
        let path = cache_dir.as_ref().join(Self::KEY);
        Self {
            shape,
            trim_steps,
            draught_steps,
            cache: Arc::new(RwLock::new(None)),
            cache_path: path,
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// See [BuildAreaCache] for details.
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        if let Err(err) = self.shape.init() {
            return vec![error.pass_with("self.shape.init()", err.to_string())];
        };
        let cache_data = super::build_cache::BuildAreaCache::new(
            &self.dbg,
            self.shape.clone(),
            self.trim_steps.clone(),
            self.draught_steps.clone(),
            self.scheduler.clone(),
            self.exit.clone(),
        )
        .build();
        let data: Vec<_> = cache_data.iter().filter_map(|v| v.clone().ok()).collect();
        let mut errors: Vec<_> = cache_data.into_iter().filter_map(|v| v.err()).collect();
        if let Some(mut guard) = self.cache.try_write() {
            let cache = if let Some(cache) = guard.take() {
                cache
            } else {
                Cache::<f64>::new(&self.dbg)
            };
            if let Err(err) = cache.init(data.clone()) {
                errors.push(error.pass_with("self.cache.get_mut", err));
            }
            let _ = guard.insert(cache);
            if let Err(err) = self.save(data) {
                errors.push(error.pass_with("save data", err));
            }
        } else {
            errors.push(error.err("self.cache.get_mut error: no cache"));
        }
        errors
    }

    ///
    /// read cache data from `self.path` file.
    ///
    /// # Panics
    /// Panic occurs if the reader produces a non-comparable value (e. g. _NaN_).
    fn read(&self) -> Result<Vec<Vec<f64>>, Error> {
        let callee = "read_from_file";
        let file = File::open(&self.cache_path).map_err(|err| {
            format!(
                "{}.{} | Failed reading file='{}': {}",
                self.dbg,
                callee,
                self.cache_path.display(),
                err
            )
        })?;
        let reader = BufReader::new(file);
        let mut vals = None;
        for (try_line, line_id) in reader.lines().zip(1..) {
            let line = try_line.map_err(|err| {
                format!(
                    "{}.{} | Failed reading line={}: {}",
                    self.dbg, callee, line_id, err
                )
            })?;
            let ss = line.split_ascii_whitespace();
            let ss_len = ss.clone().count();
            let vals_mut = match vals.as_mut() {
                None => vals.insert(vec![vec![]; ss_len]),
                Some(vals) if vals.len() != ss_len => {
                    return Err(format!(
                        "{}.{} | Inconsistent dataset at line={}",
                        self.dbg, callee, line_id
                    )
                    .into());
                }
                Some(vals) => vals,
            };
            for (i, s) in ss.enumerate() {
                let val = s.parse().map_err(|err| {
                    format!(
                        "{}.{} | Failed parsing value at line={}: {}",
                        self.dbg, callee, line_id, err
                    )
                })?;
                vals_mut[i].push(val);
            }
        }
        vals.ok_or(format!("{}.{} | Error: no vals", self.dbg, callee,).into())
    }
    ///
    /// save cache data to `self.path` file.
    ///
    fn save(&self, vals: Vec<Vec<f64>>) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "save_to_file");
        let mut file = File::create(&self.cache_path).map_err(|err| {
            error.pass_with(
                format!("File::create error! path:{}", self.cache_path.display()),
                err.to_string(),
            )
        })?;
        for col in vals.iter() {
            let cols_str: Vec<_> = col.iter().map(ToString::to_string).collect();
            let line = cols_str.join("\t");
            writeln!(&mut file, "{}", line).map_err(|err| {
                error.pass_with(
                    format!("Writing to file, path:{}", self.cache_path.display()),
                    err.to_string(),
                )
            })?;
        }
        Ok(())
    }
}
//
//
impl LocalCache for AreaCache {
    ///
    /// See [Cache::get] for details.
    fn get(&self, approx_vals: &[Option<f64>]) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "get");
        if self.cache.read().is_none() {
            let cache = Cache::new(&self.dbg);
            let vals = self
                .read()
                .map_err(|err| error.pass_with("read cache data error", err))?;
            cache
                .init(vals)
                .map_err(|err| error.pass_with("cache.init error", err))?;
            let _ = self.cache.write().insert(cache);
        }
        Ok(self
            .cache
            .read()
            .as_ref()
            .ok_or(error.pass("no cache"))?
            .get(approx_vals))
    }
    //
    //
    fn rebuild(&mut self) -> Result<(), Error> {
        self.exit.store(false, Ordering::SeqCst);
        match self.calculate().first() {
            Some(err) => Err(Error::new(&self.dbg, "rebuild").pass(err.to_owned())),
            None => Ok(()),
        }
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst)
    }
}
