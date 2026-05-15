use std::path::PathBuf;

use crate::algorithm::entities::Bounds;
use sal_3dlib::WindageProfile;
use sal_core::{dbg::Dbg, error::Error};
///
/// площади и смещения расчета остойчивости
#[derive(Debug, Clone)]
pub struct AreaResult {
    /// Площадь парусности сплошных поверхностей для осадки d_min без палубного груза, м^2
    pub av_cs_dmin: f64,
    /// Cтатический момент площади парусности по длине относительно начала координат, м^2
    pub mv_x_cs_dmin: f64,
    /// Cтатический момент площади парусности по высоте относительно ОП, м^2
    pub mv_z_cs_dmin: f64,
    /// Разница в площадях парусности для текущей осадки и осадки d_min, м^2
    pub delta_av: f64,
    /// Разница в статических моментах для текущей осадки и осадки dmin относительно начала координат, м^3
    pub delta_mv_x: f64,
    /// Разница в статических моментах для текущей осадки и осадки dmin относительно ОП, м^3
    pub delta_mv_z: f64,
    /// Отстояние по вертикали центра площади проекции подводной части корпуса на диаметральную плоскость
    /// в прямом положении судна (при нулевом крене) на спокойной воде для текущей осадки [м]
    pub area_volume_z: f64,
}
///
/// Площадь парусности корпуса и конструкций
pub struct WindageArea {
    dbg: Dbg,
    cache_dir: PathBuf,
    windage_area: Option<WindageProfile>,
    draught_min: f64,
    bounds: Option<Bounds>,
}
//
//
impl WindageArea {
    ///
    /// Creates a new instance.
    /// - cache_path - the folder contains all caches
    ///
    pub fn new(parent: &Dbg, cache_dir: PathBuf, draught_min: f64) -> Self {
        let dbg = Dbg::new(parent, "WindageArea");
        Self {
            dbg,
            cache_dir,
            windage_area: None,
            bounds: None,
            draught_min,
        }
    }
    /// инициализация заранее посчитанными данными
    pub fn init(&mut self, bounds: Bounds) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "init");
        self.windage_area = Some(
            WindageProfile::read(&self.cache_dir.join("windage")).map_err(|err| error.pass(err))?,
        );
        self.bounds = Some(bounds);
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
        let (av_cs, mv_x, mv_z, area_volume_z) = windage_area.calculate_area(draught, 0.);
        let (av_cs_dmin, mv_x_cs_dmin, mv_z_cs_dmin, _) =
            windage_area.calculate_area(self.draught_min, 0.);
        let res = AreaResult {
            av_cs_dmin,
            mv_x_cs_dmin,
            mv_z_cs_dmin,
            delta_av: av_cs_dmin - av_cs,
            delta_mv_x: mv_x_cs_dmin - mv_x,
            delta_mv_z: mv_z_cs_dmin - mv_z,
            area_volume_z,
        };
        Ok(res)
    }
    /// Расчет площади и центра площади парусности для минимальной осадки
    pub fn windage_area_min(&self) -> Result<(f64, f64, f64), Error> {
        let error = Error::new(&self.dbg, "windage_area_min");
        let windage_area = self
            .windage_area
            .as_ref()
            .ok_or(error.pass("no windage_area"))?;
        let (av_cs_dmin, mv_x_cs_dmin, mv_z_cs_dmin, _) =
            windage_area.calculate_area(self.draught_min, 0.);
        Ok((av_cs_dmin, mv_x_cs_dmin, mv_z_cs_dmin))
    }
    /// Расчет распределения площади парусности
    /// Возвращает набор значений (начало площади по x, конец площади по x, массив значений площади)
    pub fn bounded_windage_area(&self) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "bounded_windage_area");
        let windage_area = self
            .windage_area
            .as_ref()
            .ok_or(error.pass("no windage_area"))?;
        let bounds = self.bounds.as_ref().ok_or(error.pass("no bounds"))?;
        let x_max = windage_area.x_min + windage_area.step * windage_area.columns.len() as f64;
        let src_bounds =
            Bounds::from_min_max(windage_area.x_min, x_max, windage_area.columns.len())
                .map_err(|err| error.pass(err))?;
        let src_values = windage_area.calculate_area_array(self.draught_min, 0.);
        bounds
            .intersect(&src_bounds, &src_values)
            .map_err(|err| error.pass(err))
    }
    /// Расчет площади проекции по правилу дополнительного запаса плавучести в носу
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part03_draft/chapter02_draftCriteria/section04_bowBuoyancy.md]
    pub fn bow_area(&self, trim: f64, draught: f64) -> Result<f64, Error> {
        let error = Error::new(&self.dbg, "bounded_windage_area");
        let windage_area = self
            .windage_area
            .as_ref()
            .ok_or(error.pass("no windage_area"))?;
        windage_area
            .bow_area(draught, trim)
            .map_err(|err| error.pass(err))
    }
}
//
#[cfg(test)]
impl WindageArea {
    /// Создает "фейковый" объект парусности для тестов.
    /// Позволяет передать уже готовые (замоканные) кэши и значения,
    /// чтобы не производить тяжелые расчеты и дисковые операции.
    pub fn create_test_fake(
        midel_dx: f64,
        draught_min: f64,
        lbp: f64,
        values: Vec<f64>,
    ) -> Self {
        use sal_3dlib::WindageColumn;
        let x_min = midel_dx - lbp/2.;
        let x_max = midel_dx + lbp/2.;
        let bounds = Bounds::from_min_max(x_min, x_max, values.len()).unwrap();
        let step = (x_max - x_min) / values.len() as f64;
        let columns = values.iter()
                    .map(|v| WindageColumn {
                        intervals: vec![(draught_min, draught_min + v/step)],
                    })
                    .collect();
        Self {
            dbg: sal_core::dbg::Dbg::new("test", "SimpleMockWindageArea"),
            cache_dir: PathBuf::from("/tmp/test_windage_cache"),
            windage_area: Some(WindageProfile {
                x_min,
                step,
                midel_dx,
                draught_min,
                lbp,
                columns,
            }),
            draught_min,
            bounds: Some(bounds),
        }
    }
}
