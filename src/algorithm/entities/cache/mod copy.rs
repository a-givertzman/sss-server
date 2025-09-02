//!
//! Generic cache implementation.
//!
//! This implemetation can be used either directly or
//! be taken to create a more specific cache structure.
//
use sal_core::{dbg::Dbg, error::Error};
use std::{num::ParseFloatError, str::FromStr, sync::OnceLock};
//
pub struct Cache<T> {
    dbg: Dbg,
    qnt_keys: usize,
    table: OnceLock<Vec<Vec<T>>>, // (keys, values)
}
//
//
impl<T> Cache<T> {
    ///
    /// Creates a new instance.
    ///
    /// Note that this call doesn't read the file yet.
    /// The first access (see [Cache::get]) causes file reading.
    pub fn new(parent: &Dbg, qnt_keys: usize) -> Self {
        assert!(qnt_keys > 0);
        Self {
            dbg: Dbg::new(parent, "Cache"),
            qnt_keys,
            table: OnceLock::new(),
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
    /// Panic occurs if the reader produces a non-comparable value (e. g. _NaN_)
    /// qnt_keys >= vals len
    /// vals len < 2
    pub fn init(&self, vals: Vec<Vec<T>>)  -> Result<(), Error>
    where
        T: FromStr<Err = ParseFloatError> + Clone + Default + std::fmt::Display,
    {
        assert!(vals.len() > 1);
        assert!(vals[0].len() > 1);
        assert!(self.qnt_keys < vals.len());
        self.table
            .set(vals)
            .map_err(|_| Error::new("Cache", "init").err("table.set"))?;
        Ok(())
    }
}
//
//
impl Cache<f64> {
    ///
    /// # Panics
    /// Panic occurs if the reader produces a non-comparable value (e. g. _NaN_)
    /// query != keys len
    pub fn get(&self, query: &[f64]) -> Vec<f64> {
        assert_eq!(self.qnt_keys, query.len());
        dbg!(query);
        let data = self
            .table
            .get()
            .unwrap_or_else(|| panic!("{}.{} | Error: no table!", self.dbg, "get"));
        dbg!(data);
        // пары значений для каждого индекса, между которыми попадает ключ
        let pairs: Vec<_> = query
            .iter()
            .enumerate()
            .map(|(key_i, key)| {
                dbg!(key_i, key);
           /*     let mut data: Vec<_> = data
                    .iter()
                    .map(|v| (v[key_i], ((key - v[key_i]) as f64).abs()))
                    .collect();*/
                let mut data: Vec<_> = data[key_i]
                    .iter()
                    .map(|&v| (v, ((key - v) as f64).abs()))
                    .collect();
                data.sort_by(|&a, &b| a.1.partial_cmp(&b.1).unwrap());
                data.dedup();
                let res = if data[0].1 == 0. {
                    vec![data[0].0]
                } else {
                    vec![data[0].0, data[1].0]
                };
                dbg!(&data, &res);
                res
            })
            .collect();
        dbg!(&pairs);
        // фильтруем данные, оставляя только те строки, которые содержат какое-либо значение из пар
     //   dbg!(&data);

        for (i, p) in pairs.iter().enumerate() {
            let v = &data[i];
            if !v.contains(v) {
                return false;
            }
        }

        let data: Vec<_> = data
            .iter()
            .filter(|v| {
                for (i, v) in pairs.iter().enumerate() {
                    if !c.as_ref().clone().unwrap().contains(v) {
                        return false;
                    }
                }

                for (c, v) in pairs.iter().zip(v.iter()).filter(|(p, _)| p.is_some()) {
                    if !c.as_ref().clone().unwrap().contains(v) {
                        return false;
                    }
                }
                true
            })
            .collect();
        dbg!(&data);
        // расчитываем дельту для каждого индекса
        let keys_and_delta: Vec<_> = query
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let Some(key) = *key else {
                    return None;
                };
                let mut data: Vec<_> = data.iter().map(|v| v[i]).collect();
                data.sort_by(|a, b| a.partial_cmp(b).unwrap());
                data.dedup();
                //   println!("{i} {:?}", data);
                debug_assert!(data.len() > 0);
                if data.len() == 1 {
                    debug_assert_eq!(key, data[0], "{}", format!("key:{key}, data:{:?} approx_vals:{:?}", data, query));
                    Some((key, 1.)) // ключ всегда будет равен значению, дельта не важна
                } else {
                    debug_assert_eq!(data.len(), 2);
                    debug_assert!(data[0] < key && key < data[1], "{}", format!("key:{key}, data:{:?}", data));
                    Some((key, data[1] - data[0]))
                }
            })
            .collect();
        // для каждой строки считаем коэффициенты и перемножаем их на значения
        let result = data
            .iter()
            .map(|v| {
                // коэффициент для каждой строки, получается перемножением коэффициентов для каждого индекса
                let multipler = keys_and_delta
                    .iter()
                    .zip(v.iter())
                    .filter(|(k, _)| k.is_some())
                    .fold(1., |acc, (k, v)| {
                        let (key, delta) = k.unwrap();
                        acc * (1. - ((key - v) as f64).abs() / delta)
                    });
                // перемножаем каждое значение в строке на коэффициент строки, это будет
                // вклад значения строки по этому индексу в итоговое значение 
                v.iter().map(|v| v * multipler).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        // последовательно суммируем вклад строк по каждому индексу
        let result = (0..approx_vals.len())
            .map(|i| result.iter().map(|v| v[i]).sum::<f64>())
            .collect::<Vec<_>>();
        result
    }
}
