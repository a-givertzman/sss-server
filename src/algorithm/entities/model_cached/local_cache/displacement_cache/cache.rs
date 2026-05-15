use crate::algorithm::entities::{
    Position,
    Cache,
    model_cached::{DisplacementCacheResult, get_from_volume, local_cache::LocalCache},
};
use sal_core::{dbg::Dbg, error::Error};
use std::path::{Path, PathBuf};
///
/// Pre-calculated cache for floating position algorithm.
/// contains keys: [heel, trim, draught]
/// values:[volume, x, y, z, area, x, y, z, waterline_x, waterline_y]
pub struct DisplacementCache {
    dbg: Dbg,
    cache_path: PathBuf,
    cache: Option<Cache<f64>>,
}
//
//
impl DisplacementCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        cache_dir: impl AsRef<Path>,
    ) -> Self {
        let dbg = Dbg::new(parent, "DisplacementCache");
        let path = cache_dir.as_ref().join("displacement_cache");
        Self {
            cache: None,
            cache_path: path,
            dbg,
        }
    }
    /// Получение данных кэша для текущего положения
    /// Итерационно подбирает значение водоизмещения по осадке    
    pub fn get(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
    ) -> Result<DisplacementCacheResult, Error> {
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let (draught, result) = get_from_volume(
            &self.dbg,
            cache,
            &[heel, trim],
            volume,
            None,
            None,
            3,
            epsilon,
        )
        .map_err(|err| error.pass(err))?;
        Ok(DisplacementCacheResult {
            heel,
            trim,
            draught,
            volume: result[0],
            volume_center: Position::new(result[1], result[2], result[3]),
            area_wl: result[4],
            area_wl_center: Position::new(result[5], result[6], result[7]),
            inertia_trans_x: result[8],
            inertia_long_y: result[9],
            length_wl: result[10],
            breadth_wl: result[11],
        })
    }
    //
    pub fn get_volume_disp(&self) -> Result<(f64, f64), Error> {
        let error = Error::new(self.dbg(), "get_max_volume");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        Ok(cache.disp(3))
    }
}
//
impl LocalCache for DisplacementCache {
    //
    fn dbg(&self) -> &Dbg {
        &self.dbg
    }
    //
    fn cache_path(&self) -> PathBuf {
        self.cache_path.clone()
    }
    //
    fn cache(&self) -> Option<&Cache<f64>> {
        self.cache.as_ref()
    }

    fn set_cache(&mut self, cache: Cache<f64>) {
        let _ = self.cache.insert(cache);
    }
}
//
#[cfg(test)]
impl DisplacementCache {
    /// Создает "фейковый" кэш гидростатики для тестов.
    /// Позволяет задать предвычисленные значения без прогона тяжелых циклов.
    pub fn create_test_fake(mock_cache_data: Option<Cache<f64>>) -> Self {
        let dbg = Dbg::new("test", "FakeDisplacementCache");
        Self {
            dbg: dbg.clone(),
            cache_path: PathBuf::from("/tmp/test_displacement_cache"),
            cache: mock_cache_data,
        }
    }
    /// Вспомогательный метод для создания мока с уже заполненными
    /// базовыми данными для одной точки (0 крена, 0 дифферента).
    pub fn create_simple_mock(volume: f64, area_wl: f64) -> Self {
        let dbg = sal_core::dbg::Dbg::new("test", "SimpleMockCache");
        let cache = Cache::<f64>::new(&dbg);
        let fake_data = vec![
            vec![
                0.0, 0.0, 0.0, volume, // result[0]
                0.0, 0.0, -0.5,    // result[1,2,3] - Центр величины (x, y, z)
                area_wl, // result[4]
                0.0, 0.0, 0.0, // result[5,6,7] - Центр ВЛ
                100.0, 500.0, // result[8,9] - Инерция x, y
                10.0, 2.0, // result[10,11] - Длина и ширина ВЛ
            ],
            vec![
                0.0, 0.0, 100.0, volume, // result[0]
                0.0, 0.0, -0.5,    // result[1,2,3] - Центр величины (x, y, z)
                area_wl, // result[4]
                0.0, 0.0, 0.0, // result[5,6,7] - Центр ВЛ
                100.0, 500.0, // result[8,9] - Инерция x, y
                10.0, 2.0, // result[10,11] - Длина и ширина ВЛ
            ],
        ];
        let _ = cache.init(fake_data);
        Self::create_test_fake(Some(cache))
    }
}
