mod tests;
mod file_io;

use crate::{
    algorithm::entities::{
        Bounds, cache::Cache, model_cached::AreaShape
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{JoinHandle, Scheduler};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
///
///
/// Площадь парусности корпуса и конструкций
/// TODO: не зависит от крена и дифферента, поэтому не кэш, возможно надо переработать
pub struct WindageArea {
    dbg: Dbg,
    cache_path: PathBuf,
    /// Draught in meters
    draught_min: f64,
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<AreaShape>>,
    /// Cache read from `self.file_path`.
    area: Option<(f64, f64)>, //[area, center_x]
    area_data: Option<Vec<(f64, f64)>>,//распределение [area, center_x]
}
//
//
impl WindageArea {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    /// 
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<AreaShape>>,
        cache_dir: impl AsRef<Path>,
        draught_min: f64,
    ) -> Self {
        let dbg = Dbg::new(parent, "AreaCache");
        let path = cache_dir.as_ref().join("area_cache");
        Self {
            dbg,
            cache_path: path,
            draught_min,
            shape,
            area: None,
            area_data: None,
        }
    }
    //
    fn calculate(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "calculate");
        let area_data = self.shape.read().windage_area_data(self.draught_min).map_err(|err|error.pass_with("shape.windage_area_data", err))?;
        file_io::save(&self.dbg, &self.cache_path, &area_data).map_err(|err| error.pass_with("file_io::save", err))?;
        self.area_data = Some(area_data);
        Ok(())
    }
    /// Расчет площади и центра площади парусности
    /// Возвращает [площадь, смещение площади по x]
    pub fn windage_area(&mut self) -> Result<(f64, f64), Error> {
        let error = Error::new(&self.dbg, "windage_area");
        if let Some((area_sum, center_x)) = self.area.as_ref() {
            Ok((*area_sum,*center_x))
        } else {
            let area_data = if let Some(area_data) = self.area_data.as_ref() {
                area_data
            } else {            
                self.area_data = Some(file_io::read(&self.dbg, &self.cache_path).map_err(|err| error.pass_with("file_io::read", err))?);
                self.area_data.as_ref().unwrap()
            };          
            let mut area_sum = 0.;
            let mut moment = 0.;
            for (x, area) in area_data.iter() {
                moment += x * area;
                area_sum += area;
            }
            let center_x = moment / area_sum;
            self.area = Some((area_sum, center_x));
            Ok((area_sum, center_x))
        }
    }
    /// Расчет распределения площади парусности
    /// Возвращает набор значений (начало площади по x, конец площади по x, массив значений площади)
    pub fn bounded_windage_area(
        &mut self,
        bounds: Bounds,
    ) -> Result<(f64, f64, Vec<f64>), Error> {
        let error = Error::new(&self.dbg, "bounded_windage_area");
        // набор значений площади в разбиении по площади части модели над водой
        let area_data = if let Some(area_data) = self.area_data.as_ref() {
            area_data
        } else {            
            self.area_data = Some(file_io::read(&self.dbg, &self.cache_path).map_err(|err| error.pass_with("file_io::read", err))?);
            self.area_data.as_ref().unwrap()
        };  
        let x_min = area_data.first().ok_or(error.err("empty result from _windage_area"))?.0;
        let x_max = area_data.last().ok_or(error.err("empty result from _windage_area"))?.0;
        let dx = (x_max - x_min) / 2. * ((area_data.len() - 1) as f64);
        let (min, max, n) = (x_min - dx, x_max + dx, area_data.len());
        let bounds = Bounds::from_min_max(min, max, n)
            .map_err(|err| error.pass_with(format!("Bounds::from_min_max min:{min}, max:{max}, n:{n}"), err))?;
        let values = 
            bounds.iter().zip(&area_data.iter().map(|v| v.0).collect());


                let mut current_max_x = 0;
                let mut result = Vec::new();
                let mut current = Vec::new();
                for v in values {
                    if p.coords.x > current_max_x {
                        current.sort();
                        current.dedup();
                        result.push((x(current_max_x), current.iter().map(|&v| z(v) ).collect()));
                        current = Vec::new();
                        current_max_x += 1;
                        while p.coords.x > current_max_x {
                            result.push((x(current_max_x), Vec::new()));
                            current_max_x += 1;                            
                        }
                    }
                    current.push(p.coords.z);
                }

        Ok((
            x_min - dx,
            x_max + dx,
            area_data.into_iter().map(|(_, area)| area).collect(),
        ))
    }
}
