use crate::{algorithm::entities::{cache::{self, Cache}, model::{local_cache::LocalCache, ModelTree}, Position}, kernel::types::RwLock};
use sal_3dlib::topology::shape::{
    face::Face,
    vertex::Vertex,
    wire::{Polygon, Wire},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::Scheduler;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, Arc},
};

use super::{build_displacement_cache::BuildDisplacementCache, DisplacementCacheConf};
///
/// Pre-calculated cache for floating position algorithm.
///
/// See [DisplacementCacheConf] for more details about the fields.
pub struct DisplacementCache {
    dbg: Dbg,
    path: PathBuf,
    //    model_keys: Vec<String>,
    waterline_position: Position,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    ///
    /// Model representation used for cache calculation.
    model_tree: ModelTree,
    ///
    /// Cache read from `self.file_path`.
    cache: Arc<RwLock<Option<Cache<f64>>>>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl DisplacementCache {
    //
    //
    const KEY: &'static str = "floating_position_cache";
    ///
    /// Creates a new instance.
    /// - path - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        model_tree: ModelTree,
        path: impl AsRef<Path>,
        conf: DisplacementCacheConf,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, "DisplacementCache");
        let path = path.as_ref().join(Self::KEY);
        Self {
            model_tree,
            //         model_keys: vec![],
            heel_steps: conf.heel_steps,
            waterline_position: conf.waterline_position,
            trim_steps: conf.trim_steps,
            draught_steps: conf.draught_steps,
            cache: Arc::new(RwLock::new(None)),
            path,
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// See [BuildDisplacementCache] for details.
    fn calculate(&self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let model_tree = self.model_tree.clone();
        let model_tree = match model_tree.load() {
            Ok(model_tree) => model_tree,
            Err(err) => {
                return vec![error.pass_with("model_tree", err)];
            }
        };
        let cache_data = BuildDisplacementCache::new(
                &self.dbg,
                model_tree.iter().map(|(_, shape)| shape).cloned().collect(),
                self.waterline_position,
                /* TODO зачем этот фильтр?
                        .iter()
                        .filter_map(|(shape_key, shape)| {
                                self.model_keys.contains(shape_key).then_some(shape)
                            })
                        .cloned()
                            .collect(),
                */
                self.heel_steps.clone(),
                self.trim_steps.clone(),
                self.draught_steps.clone(),
                self.scheduler.clone(),
                self.exit.clone(),
            )
            .build();
        let data: Vec<_> = 
            cache_data
            .iter()
            .filter_map(|v| v.clone().ok())
            .collect(); 
        let mut errors: Vec<_> = 
            cache_data
            .into_iter()
            .filter_map(|v| v.err())
            .collect(); 
        if let Some(cache) = self.cache.write().as_ref() {
            if let Err(err) = cache.init(data.clone()) {
                errors.push(error.pass_with("self.cache.get_mut", err));
            }
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
    fn read(&self) -> Result<Vec<Vec<f64>>, Error>
    {
        let callee = "read_from_file";
        let file = File::open(&self.path).map_err(|err| {
            format!(
                "{}.{} | Failed reading file='{}': {}",
                self.dbg,
                callee,
                self.path.display(),
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
        vals.ok_or(                
            format!(
            "{}.{} | Error: no vals",
            self.dbg, callee,
        ).into())
    }
    ///
    /// save cache data to `self.path` file.
    ///
    fn save(&self, vals: Vec<Vec<f64>>) -> Result<(), Error>
    {
        let error = Error::new(&self.dbg, "save_to_file");
        let mut file = File::create(&self.path).map_err(|err| {
            error.pass_with(
                format!("File::create error! path:{}", self.path.display()),
                err.to_string(),
            )
        })?;
        for col in vals.iter() {
            let cols_str: Vec<_> = col.iter().map(ToString::to_string).collect();
            let line = cols_str.join("\t");
            writeln!(&mut file, "{}", line).map_err(|err| error.pass_with(
                format!("Writing to file, path:{}", self.path.display()), 
                err.to_string(),
            ))?;
        }
        Ok(())
    }
}
//
//
impl LocalCache for DisplacementCache {
    ///
    /// See [Cache::get] for details.
    fn get(&self, approx_vals: &[Option<f64>]) -> Result<Vec<Vec<f64>>, Error> {
        let error = Error::new(&self.dbg, "get");
        if self.cache.read().is_none() {
            let cache = Cache::new(&self.dbg);
            let vals = self.read()
            .map_err(|err| 
                error.pass_with("read cache data error", err)
            )?;
            cache.init(vals).map_err(|err| 
                error.pass_with("cache.init error", err)
            )?;
            self.cache.write().insert(cache);
        }
        self.cache
            .read()
            .as_ref()
            .ok_or(error.pass("no cache"))?
            .get(approx_vals)
            .ok_or(error.pass("can't get values from cache"))
    }
    //
    //
    fn rebuild(&self) -> Result<(), Error> {
        self.exit.store(false, Ordering::SeqCst);
        match self.calculate().first() {
            Some(err) => Err(Error::new(&self.dbg, "rebuild").pass(err.to_owned())),
            None => {
                Ok(())
            }
        }
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst)
    }
}
