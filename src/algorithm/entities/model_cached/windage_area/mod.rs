mod file_io;
mod tests;

use crate::{
    algorithm::entities::{
        Bounds,
        model_cached::{AreaCache, AreaCacheResult, AreaData, AreaShape, BowAreaCache, LocalCache},
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use std::path::PathBuf;
///
///
/// Площадь парусности корпуса и конструкций
/// TODO: не зависит от крена и дифферента, поэтому не кэш, возможно надо переработать
pub struct WindageArea {
    dbg: Dbg,
    cache_dir: PathBuf,
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<AreaShape>>,
    /// - cache for windage area
    windage_area: Option<AreaCache>,
    /// - cache for bow area
    bow_area: Option<BowAreaCache>,
    /// Cache read from `self.file_path`.
    values: Option<Vec<f64>>, //распределение
    draught_min: f64,
    thread_pool: Arc<ThreadPool>,
}
//
//
impl WindageArea {
    ///
    /// Creates a new instance.
    /// - cache_path - the folder contains all caches
    ///
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<AreaShape>>,
        cache_dir: PathBuf,
        draught_min: f64,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindageArea");
        Self {
            dbg,
            cache_dir: cache_dir,
            shape,
            windage_area: None,
            bow_area: None,
            values: None,
            draught_min,
            thread_pool,
        }
    }
    //
    pub fn rebuild(&mut self, bounds: &Bounds) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "calculate");
        let area_data = self
            .shape
            .read()
            .windage_area_data()
            .map_err(|err| error.pass_with("shape.windage_area_data", err))?;
        let cache_path = self.cache_dir.join("windage_area");
        file_io::save(&self.dbg, &cache_path, area_data)
            .map_err(|err| error.pass_with("file_io::save", err))?;
        self.init(bounds)
    }
    /// инициализация заранее посчитанными данными
    pub fn init(&mut self, bounds: &Bounds) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "init");
        let cache_path = self.cache_dir.join("windage_area");
        let AreaData {
            x_start,
            x_end,
            voxels,
        } = file_io::read(&self.dbg, &cache_path)
            .map_err(|err| error.pass_with("file_io::read", err))?;
        let area_data = Arc::new(voxels);
        let mut windage_area = AreaCache::new(
            &self.dbg,
            self.draught_min,
            Arc::clone(&area_data),
            self.cache_dir.clone(),
            Arc::clone(&self.thread_pool),
        );
        let errors = windage_area.calculate();
        if !errors.is_empty() {
            return Err(error.pass_with(
                " windage_area.calculate",
                errors
                    .iter()
                    .fold(String::new(), |acc, err| acc + &format!(" error: {err}")),
            ));
        }
        self.windage_area = Some(windage_area);
        let mut bow_area = BowAreaCache::new(
            &self.dbg,
            self.draught_min,
            Arc::clone(&area_data),
            self.cache_dir.clone(),
            Arc::clone(&self.thread_pool),
        );
        self.bow_area = Some(bow_area);
        let mut area_sum = 0.;
        let (mut moment_x, mut moment_z) = (0., 0.);
        let area_data: Vec<_> = area_data
            .iter()
            .map(|(x, v)| {
                let a = v.iter().map(|(_, a)| a).sum();
                area_sum += a;
                moment_x += x * a;
                v.into_iter().for_each(|(z, a)| moment_z += a * z);
                (x, a)
            })
            .collect();
        let src_bounds = Bounds::from_min_max(x_start, x_end, area_data.len()).map_err(|err| {
            error.pass_with(
                format!(
                    "Bounds::from_min_max x_start:{x_start}, x_end:{x_end}, n:{}",
                    area_data.len()
                ),
                err,
            )
        })?;
        let src_values: Vec<f64> = area_data.into_iter().map(|(_, v)| v).collect();
        self.values = Some(
            bounds
                .intersect(&src_bounds, &src_values)
                .map_err(|err| error.pass_with("bounds.intersect", err))?,
        );
        Ok(())
    }
    /// Расчет площади и центра площади парусности
    /// Возаращает (area_windage, area_windage_z, delta_area_windage, area_volume_z)
    pub fn windage_area(&self, draught: f64) -> Result<(f64, f64, f64, f64), Error> {
        let error = Error::new(&self.dbg, "windage_area");
        let windage_area = self
            .windage_area
            .as_ref()
            .ok_or(error.pass("no windage_area"))?;
        let AreaCacheResult {
            area_windage,
            area_windage_z,
            delta_area_windage,
            area_volume_z,
        } = windage_area.get(draught).map_err(|err| error.pass(err))?;
        Ok((
            area_windage,
            area_windage_z,
            delta_area_windage,
            area_volume_z,
        ))
    }
    /// Расчет распределения площади парусности
    /// Возвращает набор значений (начало площади по x, конец площади по x, массив значений площади)
    pub fn bounded_windage_area(&self) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "bounded_windage_area");
        // набор значений площади в разбиении по площади части модели над водой
        self.values.clone().ok_or(error.pass("no values"))
    }

    ///
    pub fn bow_area(&self, trim: f64) -> Result<f64, Error> {
        TODO
    }
}
