mod file_io;
//mod tests;

use crate::{
    algorithm::entities::{
        Bounds,
        model_cached::{
            AreaCache, AreaData, AreaResult, AreaShape, BowAreaCache, LocalCache, Shape
        },
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use std::path::PathBuf;
///
///
/// Площадь парусности корпуса и конструкций
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
            cache_dir,
            shape,
            windage_area: None,
            bow_area: None,
            values: None,
            draught_min,
            thread_pool,
        }
    }
    /// пересчет для заданных значений
    pub fn rebuild(&mut self, bounds: &Bounds, lbp: f64) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "calculate");
        let shape = self.shape.read();
        let AreaData {
            x_start,
            x_end,
            voxels,
        } = shape
            .windage_area_data()
            .map_err(|err| error.pass_with("shape.windage_area_data", err))?;
        let center_x = shape.center().ok_or(error.err("shape.center"))?.x;
        let mut windage_area = AreaCache::new(
            &self.dbg,
            self.draught_min,
            Some(voxels.clone()),
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
        let bow_area_voxels = get_bow_area(&voxels, x_start, x_end, center_x, lbp);
        let mut bow_area = BowAreaCache::new(
            &self.dbg,
            Some(bow_area_voxels),
            Some((x_end - x_start) / voxels.len() as f64),
            &self.cache_dir,
            Arc::clone(&self.thread_pool),
        );
        let errors = bow_area.calculate();
        if !errors.is_empty() {
            return Err(error.pass_with(
                " bow_area.calculate",
                errors
                    .iter()
                    .fold(String::new(), |acc, err| acc + &format!(" error: {err}")),
            ));
        }
        self.bow_area = Some(bow_area);
        let values = get_bounds_area(&self.dbg, x_start, x_end, &voxels, bounds)
            .map_err(|err| error.pass(err))?;
        file_io::save(
            &self.dbg,
            &self.cache_dir.join("bounded_windage_area"),
            &values,
        )
        .map_err(|err| error.pass_with("file_io::save", err))?;
        self.values = Some(values);
        Ok(())
    }
    /// инициализация заранее посчитанными данными
    pub fn init(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "init");
        let mut windage_area = AreaCache::new(
            &self.dbg,
            self.draught_min,
            None,
            self.cache_dir.clone(),
            Arc::clone(&self.thread_pool),
        );
        windage_area.init().map_err(|err| error.pass(err))?;
        self.windage_area = Some(windage_area);
        let mut bow_area = BowAreaCache::new(
            &self.dbg,
            None,
            None,
            &self.cache_dir,
            Arc::clone(&self.thread_pool),
        );
        bow_area.init().map_err(|err| error.pass(err))?;
        self.bow_area = Some(bow_area);
        self.values = Some(
            file_io::read(&self.dbg, &self.cache_dir.join("bounded_windage_area"))
                .map_err(|err| error.pass(err))?,
        );
        Ok(())
    }
    /// Расчет площади и центра площади парусности
    /// Возаращает (area_windage, area_windage_z, delta_area_windage, area_volume_z)
    pub fn windage_area(&self, draught: f64) -> Result<AreaResult, Error> {
        let error = Error::new(&self.dbg, "windage_area");
        let windage_area = self
            .windage_area
            .as_ref()
            .ok_or(error.pass("no windage_area"))?;
        let res = windage_area.get(draught).map_err(|err| error.pass(err))?;
        Ok(res)
    }
    /// Расчет площади и центра площади парусности для минимальной осадки
    pub fn windage_area_min(&self) -> Result<(f64, f64, f64), Error> {
        let error = Error::new(&self.dbg, "windage_area_min");
        let windage_area = self
            .windage_area
            .as_ref()
            .ok_or(error.pass("no windage_area"))?;
        let res = windage_area.get_min().map_err(|err| error.pass(err))?;
        Ok(res)
    }
    /// Расчет распределения площади парусности
    /// Возвращает набор значений (начало площади по x, конец площади по x, массив значений площади)
    pub fn bounded_windage_area(&self) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "bounded_windage_area");
        // набор значений площади в разбиении по площади части модели над водой
        self.values.clone().ok_or(error.pass("no values"))
    }
    /// Расчет площади проекции по правилу дополнительного запаса плавучести в носу
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part03_draft/chapter02_draftCriteria/section04_bowBuoyancy.md]
    pub fn bow_area(&self, trim: f64, draught: f64) -> Result<f64, Error> {
        let error = Error::new(&self.dbg, "bounded_windage_area");
        // набор значений площади в разбиении по площади части модели над водой
        self.bow_area
            .as_ref()
            .ok_or(error.pass("no bow_area"))?
            .get(trim, draught)
            .map_err(|err| error.pass(err))
    }
}

/// Расчет площади проекции по правилу дополнительного запаса плавучести в носу
/// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part03_draft/chapter02_draftCriteria/section04_bowBuoyancy.md]
/// Возвращает набор вокселей для кэша обрезанных по длине по длине в корме 0.15LBP от носового перпендикуляра
/// и в нос носовым перпендикуляром
fn get_bow_area(
    voxels: &Vec<(f64, Vec<(f64, f64)>)>,
    x_start: f64,
    x_end: f64,
    center_x: f64,
    lbp: f64,
) -> Vec<(f64, Vec<(f64, f64)>)> {
    assert!(!voxels.is_empty());
    let voxel_scale = (x_end - x_start) / voxels.len() as f64;
    let len_start = lbp * 0.85;
    let len_end = lbp;
    let len_start_l = len_start - voxel_scale / 2.;
    let len_start_h = len_start + voxel_scale / 2.;
    let len_end_l = len_end - voxel_scale / 2.;
    let len_end_h = len_end + voxel_scale / 2.;
    
    voxels
        .iter()
        .filter(|&&(x, _)| x > len_start_l && x < len_end_h)
        .map(|&(x, ref v)| {
            let v = v.clone();
            let v = if x < len_start_h {
                let coef = (len_start_h - x) / voxel_scale;
                v.into_iter().map(|(z, a)| (z, a * coef)).collect()
            } else if x > len_end_l {
                let coef = (x - len_end_l) / voxel_scale;
                v.into_iter().map(|(z, a)| (z, a * coef)).collect()
            } else {
                v
            };
            (x - center_x, v)
        })
        .collect()
}
/// Пересчет вокселей в распределение суммарных площадей по х
fn get_bounds_area(
    parent: &Dbg,
    x_start: f64,
    x_end: f64,
    voxels: &Vec<(f64, Vec<(f64, f64)>)>,
    bounds: &Bounds,
) -> Result<Vec<f64>, Error> {
    let error = Error::new(parent, "get_bounds_area");
    let mut area_sum = 0.;
    let (mut moment_x, mut moment_z) = (0., 0.);
    let area_data: Vec<_> = voxels
        .iter()
        .map(|(x, v)| {
            let a = v.iter().map(|(_, a)| a).sum();
            area_sum += a;
            moment_x += x * a;
            v.iter().for_each(|(z, a)| moment_z += a * z);
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
    bounds
        .intersect(&src_bounds, &src_values)
        .map_err(|err| error.pass_with("bounds.intersect", err))
}
