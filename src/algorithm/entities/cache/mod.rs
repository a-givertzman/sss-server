//!
//! Generic cache implementation.
//!
//! This implemetation can be used either directly or
//! be taken to create a more specific cache structure.
//
use sal_core::{dbg::Dbg, error::Error};
use std::{num::ParseFloatError, ops::Index, str::FromStr, sync::OnceLock};
//
type SyncVec<T> = std::sync::Arc<[T]>;
///
/// Cached dataset lazyly read from the file on the first access.
///
/// # Examples
/// ```
/// use sal_sync::services::entity::dbg_id::Dbg;
/// //
/// // only initializing, no file reading
/// let Dbg = Dbg("cache creator".to_owned());
/// let file_path = "/path/to/cache/file";
/// let cache = Cache::new(&Dbg, file_path);
/// // the first call causes reading file
/// let _ = cache.get(&[None, Some(1.0)]);
/// // the second call uses taken dataset
/// let _ = cache.get(&[Some(2.0)]);
/// ```
pub struct Cache<T> {
    dbg: Dbg,
    //таблица данных
    table: OnceLock<Vec<Vec<T>>>,
    //отсортированные вектора ключей, заполняются из таблицы при инициализации
    keys: OnceLock<Vec<Vec<T>>>, 
}
//
//
impl<T> Cache<T> {
    ///
    /// Creates a new instance.
    ///
    /// Note that this call doesn't read the file yet.
    /// The first access (see [Cache::get]) causes file reading.
    pub fn new(parent: &Dbg) -> Self {
        Self {
            dbg: Dbg::new(parent, "Cache"),
            table: OnceLock::new(),
            keys: OnceLock::new(),
        }
    }
}
//
//
impl<T: PartialOrd> Cache<T> {
    ///
    /// Initializes Table with cache data
    ///
    /// # Panics
    /// Panic occurs if the reader produces a non-comparable value (e. g. _NaN_).
    /// qnt_keys >= vals len
    /// vals len < 2
    pub fn init(&self, vals: Vec<Vec<T>>) -> Result<(), Error>
    where
        T: FromStr<Err = ParseFloatError> + Clone + Default + std::fmt::Display,
    {
        assert!(vals.len() > 1);
        assert!(vals[0].len() > 1);
        let keys: Vec<_> = (0..vals[0].len()).map(|i| {
            // выбираем столбец по его индексу
            let mut data: Vec<_> = vals.iter().map(|v| v[i].clone()).collect();
            data.sort_by(|a, b| a.partial_cmp(&b).unwrap());
            data.dedup();
            data
        }).collect();
        self.table
            .set(vals.clone())
            .map_err(|_| Error::new("Cache", "init").err("table.set"))?;
        self.keys
            .set(keys)
            .map_err(|_| Error::new("Cache", "init").err("keys.set"))?;
        Ok(())
    }
}
//
//
impl Cache<f64> {
    ///
    /// Returns approximated values based on given set.
    ///
    /// This is a safe method in terms of bounds: If `approx_vals` has more elements than [Cache] supports,
    /// this method returns `None`. In contrast, the empty vector returns if no value found.
    ///
    /// # Panics
    /// This method panics if at least one of the statements is true:
    /// - self.table not init
    /// - `approx_vals` contains a non-comparable value (e. g. _NaN_),
    ///
    /// # Examples
    /// ```
    /// fn explaination(cache: Cache<f64>) {
    ///     // get all rows of the file behind `cache`
    ///     let _ = cache.get(&[]);
    ///     // get approximated (or equal) rows, which values are calculated
    ///     // as the average of each columns between top and low bounds
    ///     // (the bounds are selected only for the first column):
    ///     // *cache file*
    ///     // |  ...     |
    ///     // |  0.0 ... | <-- top bound row
    ///     // | (0.5)    | <-- given value
    ///     // |  1.0 ... | <-- low bound row
    ///     // |  ...     |
    ///     // ------------
    ///     // ... - one or more values of type f64
    ///     let _ = cache.get(&[Some(0.5)]);
    ///     // similar to the the previous example,
    ///     // but the bounds are selected for the 2nd and 4th columns:
    ///     // *cache file*
    ///     // | *  ... ... ...  ... |
    ///     // | *  0.0  *  0.1  ... | <-- top bound row
    ///     // |   (0.1)   (0.2)     | <-- given values
    ///     // | *  1.0  *  0.5  ... | <-- low bound row
    ///     // | *  ... ... ...  ... |
    ///     // -----------------------
    ///     // * - any value of type f64
    ///     let _ = cache.get(&[None, Some(0.1), None, Some(0.2)]);
    /// }
    /// ```
    /// # Panics
    /// non-comparable value (e. g. _NaN_)
    /// qnt_keys >= vals len
    /// value is out of range
    /// value index is out of key index range - TODO описать подробнее
    pub fn get(&self, query: &[f64]) -> Vec<f64> {
        let data = self
            .table
            .get()
            .unwrap_or_else(|| panic!("{}.{} | Error: no table!", self.dbg, "get"));
        let keys = self
            .keys
            .get()
            .unwrap_or_else(|| panic!("{}.{} | Error: no keys!", self.dbg, "get"));
        // пары значений для каждого индекса, между которыми попадает ключ
        let pairs: Vec<_> = query
            .iter()
            .enumerate()
            .map(|(key_i, key)| {
                let keys = &keys[key_i];
                if keys.contains(key) {
                    // ключ совпадает с одним из значений, возвращаем его
                    return vec![*key];
                }
                if keys.first().unwrap() > key || keys.last().unwrap() < key {
                    // ключ вышел за пределы значений
                    panic!("{}", format!("i:{key_i} key:{key} key is out of range!"));
                }
                // пара значений, между которыми попадает ключ
                let low_index = keys.partition_point(|x| x < &key);
                return vec![keys[low_index-1], keys[low_index]];
            })
            .collect();
       // println!("{:?}", pairs);
        // фильтруем данные, оставляя только те строки, которые содержат какое-либо значение из пар
        let data: Vec<_> = data
            .iter()
            .filter(|v| {
                for (c, v) in pairs.iter().zip(v.iter()) {
                    if !c.contains(v) {
                        return false;
                    }
                }
                true
            })
            .collect();
        // расчитываем дельту для каждого индекса
        let keys_and_delta: Vec<_> = query
            .iter()
            .enumerate()
            .map(|(i, &key)| {
                let mut data: Vec<_> = data.iter().map(|v| v[i]).collect();
                data.sort_by(|a, b| a.partial_cmp(b).unwrap());
                data.dedup();
                debug_assert!(data.len() > 0);
                if data.len() == 1 {
                    debug_assert_eq!(
                        key,
                        data[0],
                        "{}",
                        format!("key:{key}, data:{:?} query:{:?}", data, query)
                    );
                    // если одно значение ключ всегда будет равен значению, дельта не важна
                    Some((key, 1.)) 
                } else {
                    debug_assert_eq!(data.len(), 2);
                    debug_assert!(
                        data[0] < key && key < data[1],
                        "{}",
                        format!("key:{key}, data:{:?}", data)
                    );
                    Some((key, data[1] - data[0]))
                }
            })
            .collect();
        // для каждой строки считаем коэффициенты и перемножаем их на значения
        let result = data
            .iter()
            .map(|data| {
                // коэффициент для каждой строки, получается перемножением коэффициентов для каждого индекса
                let multipler = keys_and_delta
                    .iter()
                    .zip(data.iter())
                    .filter(|(k, _)| k.is_some())
                    .fold(1., |acc, (k, data)| {
                        let (key, delta) = k.unwrap();
                        //        let k = ((key - data) as f64).abs() / delta;
                        //     dbg!(key, delta, data, k);
                        acc * (1. - ((key - data) as f64).abs() / delta)
                    });
                // перемножаем каждое значение в строке на коэффициент строки, это будет
                // вклад значения строки по этому индексу в итоговое значение
                data.iter().map(|v| v * multipler).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        // последовательно суммируем вклад строк по каждому индексу
        let result = (query.len()..result[0].len())
            .map(|i| result.iter().map(|v| v[i]).sum::<f64>())
            .collect::<Vec<_>>();
        result
    }
    pub fn max_value(&self, index: usize) -> f64 {
        let data = self
            .table
            .get()
            .unwrap_or_else(|| panic!("{}.{} | Cache error: no table!", self.dbg, "max_value"));
        assert!(data[0].len() > index);
        let v: Vec<_> = data.iter().map(|v| v[index]).collect();
        assert!(v.len() > 0);
        let v = v.into_iter().max_by(|&a, b| a.partial_cmp(b).unwrap());
        v.unwrap()
    }
}
