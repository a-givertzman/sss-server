//!
//! Generic cache implementation.
//!
//! This implemetation can be used either directly or
//! be taken to create a more specific cache structure.
//
use sal_core::{dbg::Dbg, error::Error};
use std::{num::ParseFloatError, str::FromStr, sync::OnceLock};
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
    // таблица данных в виде одного непрерывного вектора
    flat_table: OnceLock<Vec<T>>,
    // отсортированные вектора ключей, заполняются из таблицы при инициализации
    keys: OnceLock<Vec<Vec<T>>>,
    // Минимальное и максимальное значение для каждого столбца исходной таблицы
    columns_ranges: OnceLock<Vec<(T, T)>>,
    // Шаги для вычисления индекса строки в плоском массиве
    strides: OnceLock<Vec<usize>>,
    // Количество колонок во всей исходной таблице
    cols_count: OnceLock<usize>,
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
            flat_table: OnceLock::new(),
            keys: OnceLock::new(),
            columns_ranges: OnceLock::new(),
            strides: OnceLock::new(),
            cols_count: OnceLock::new(),
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
    pub fn init(&self, mut vals: Vec<Vec<T>>) -> Result<(), Error>
    where
        T: FromStr<Err = ParseFloatError> + Clone + Default + std::fmt::Display,
    {
        assert!(vals.len() > 1);
        let cols_count = vals[0].len();
        assert!(cols_count > 1);

        // 1. Извлекаем уникальные значения по всем колонкам таблицы
        let all_keys: Vec<Vec<T>> = (0..cols_count)
            .map(|i| {
                let mut data: Vec<_> = vals.iter().map(|v| v[i].clone()).collect();
                data.sort_by(|a, b| a.partial_cmp(b).unwrap());
                data.dedup();
                data
            })
            .collect();

        // 2. Сразу формируем пары (минимум, максимум) для каждого столбца
        let ranges: Vec<(T, T)> = all_keys
            .iter()
            .map(|col_data| {
                assert!(!col_data.is_empty(), "Table column cannot be empty");
                (col_data.first().unwrap().clone(), col_data.last().unwrap().clone())
            })
            .collect();

        self.columns_ranges
            .set(ranges)
            .map_err(|_| Error::new("Cache", "init").err("columns_ranges error"))?;

        // 3. Автоматический анализ: вычисляем, сколько первых колонок образуют декартову сетку
        let mut detected_keys_count = 0;
        let mut current_product = 1;
        let total_rows = vals.len();

        for i in 0..cols_count {
            current_product *= all_keys[i].len();
            if current_product == total_rows {
                detected_keys_count = i + 1;
                break;
            }
        }

        if detected_keys_count == 0 || detected_keys_count >= cols_count {
            return Err(Error::new("Cache", "init").err(
                "Failed to detect valid ND-Grid. Ensure keys are at the beginning and form a complete Cartesian product."
            ));
        }

        // Усекаем all_keys, оставляя только оси-ключи для метода get
        let mut keys = all_keys;
        keys.truncate(detected_keys_count);

        // 4. Сортируем исходную таблицу лексикографически по колонкам-ключам
        vals.sort_by(|a, b| {
            for i in 0..detected_keys_count {
                match a[i].partial_cmp(&b[i]) {
                    Some(std::cmp::Ordering::Equal) => continue,
                    other => return other.unwrap_or(std::cmp::Ordering::Equal),
                }
            }
            std::cmp::Ordering::Equal
        });

        // 5. Рассчитываем многомерные шаги (strides) для осей-ключей
        let mut strides = vec![1; detected_keys_count];
        for i in (0..detected_keys_count - 1).rev() {
            strides[i] = strides[i + 1] * keys[i + 1].len();
        }

        let flat_table: Vec<T> = vals.into_iter().flatten().collect();

        self.flat_table
            .set(flat_table)
            .map_err(|_| Error::new("Cache", "init").err("flat_table error"))?;
        self.keys
            .set(keys)
            .map_err(|_| Error::new("Cache", "init").err("keys error"))?;
        self.strides
            .set(strides)
            .map_err(|_| Error::new("Cache", "init").err("strides error"))?;
        self.cols_count
            .set(cols_count)
            .map_err(|_| Error::new("Cache", "init").err("cols_count error"))?;
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
    /// non-comparable value (e. g. _NaN_)
    /// qnt_keys >= vals len
    /// key is out of range
    /// query.len() > keys.len()
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
    pub fn get(&self, query: &[f64]) -> Vec<f64> {
        let keys = self
            .keys
            .get()
            .unwrap_or_else(|| panic!("{}.get | Error: no keys!", self.dbg));
        let flat_table = self
            .flat_table
            .get()
            .unwrap_or_else(|| panic!("{}.get | Error: no table!", self.dbg));
        let strides = self
            .strides
            .get()
            .unwrap_or_else(|| panic!("{}.get | Error: no strides!", self.dbg));
        let cols_count = *self
            .cols_count
            .get()
            .unwrap_or_else(|| panic!("{}.get | Error: no cols_count!", self.dbg));

        let query_len = query.len();
        let total_keys_count = keys.len();

        // Гарантия контракта: query не должен превышать максимальное число ключей
        assert!(
            query_len <= total_keys_count,
            "Query length cannot exceed detected keys count"
        );

        let mut low_indices = [0usize; 16];
        let mut weights = [0.0f64; 16];
        let mut is_single_val = [false; 16];

        assert!(
            query_len <= 16,
            "Dimensions higher than 16 are not supported by stack buffer"
        );

        // Шаг 1: Бинарный поиск интервалов по активным осям запроса за O(D * log K)
        for i in 0..query_len {
            let axis = &keys[i];
            let q = query[i];

            let first = *axis.first().unwrap();
            let last = *axis.last().unwrap();

            if q <= first {
                low_indices[i] = 0;
                weights[i] = 0.0;
                is_single_val[i] = true;
            } else if q >= last {
                low_indices[i] = axis.len() - 1;
                weights[i] = 0.0;
                is_single_val[i] = true;
            } else {
                let idx = axis.partition_point(|&x| x < q);

                if axis[idx] == q {
                    low_indices[i] = idx;
                    weights[i] = 0.0;
                    is_single_val[i] = true;
                } else {
                    let low_idx = idx - 1;
                    low_indices[i] = low_idx;
                    let delta = axis[idx] - axis[low_idx];
                    weights[i] = (q - axis[low_idx]) / delta;
                    is_single_val[i] = false;
                }
            }
        }

        // Подготовка результирующего вектора
        let output_len = cols_count - total_keys_count;
        let mut output = vec![0.0; output_len];

        // Шаг 2: Обход вершин окружающего гиперкуба (2^query_len итераций)
        let num_vertices = 1 << query_len;

        for v in 0..num_vertices {
            let mut row_weight = 1.0;
            let mut target_row_idx = 0;
            let mut skip_vertex = false;

            for i in 0..query_len {
                let bit = (v >> i) & 1;

                if is_single_val[i] {
                    if bit == 1 {
                        skip_vertex = true;
                        break;
                    }
                    target_row_idx += low_indices[i] * strides[i];
                    row_weight *= 1.0;
                } else {
                    if bit == 1 {
                        target_row_idx += (low_indices[i] + 1) * strides[i];
                        row_weight *= weights[i];
                    } else {
                        target_row_idx += low_indices[i] * strides[i];
                        row_weight *= 1.0 - weights[i];
                    }
                }
            }

            if skip_vertex || row_weight <= 0.0 {
                continue;
            }

            // Корректный расчет смещения для N-мерного куба со strides
            let row_start = target_row_idx * cols_count;

            // Заполнение оставшихся столбцов (начиная со смещения query_len)
            for out_idx in 0..output_len {
                output[out_idx] += flat_table[row_start + total_keys_count + out_idx] * row_weight;
            }
        }

        output
    }
    /*  /// Максимальное значение по индексу
    pub fn value_disp(&self, index: usize) -> (f64, f64) {
        let data = self.table.get().unwrap_or_else(|| {
            panic!(
                "{}.{} | Cache error: no table!",
                self.dbg, "value_disp"
            )
        });
        assert!(data[0].len() > index);
        let v: Vec<_> = data.iter().map(|v| v[index]).collect();
        assert!(v.len() > 0);
        let v_min = v.iter().min_by(|a, b| a.partial_cmp(b).unwrap());
        let v_max = v.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
        (v_min.unwrap().clone(), v_max.unwrap().clone())
    }*/
    /// Вектор значений по индексам c условием, возвращает значения только для существующих ключей
    /// ключи не должны содержать индексы значений
    /// длина query должна соответствовать количеству ключей
    /// Возвращает Vec<(value from index1, value from index2)>
    pub fn values_disp(&self, query: &[Option<f64>]) -> Vec<Vec<f64>> {
        let keys = self
            .keys
            .get()
            .unwrap_or_else(|| panic!("{}.values_disp | Error: no keys!", self.dbg));
        let total_keys_count = keys.len();

        assert_eq!(
            query.len(),
            total_keys_count,
            "Query mask length must match total keys count"
        );

        let mut current_query = vec![0.0; total_keys_count];
        let mut res = Vec::new();
        let mut i = 0;

        loop {
            let mut is_cancel = true;

            for (axis_idx, mask_val) in query.iter().enumerate() {
                let axis_keys = &keys[axis_idx];

                match mask_val {
                    Some(exact_key) => {
                        current_query[axis_idx] = *exact_key;
                    }
                    None => {
                        if axis_keys.len() <= i + 1 {
                            current_query[axis_idx] = *axis_keys.last().unwrap();
                        } else {
                            is_cancel = false;
                            current_query[axis_idx] = axis_keys[i];
                        }
                    }
                }
            }

            res.push(self.get(&current_query));

            if is_cancel {
                break;
            }
            i += 1;
        }

        res
    }
    /// Максимальное значение по индексу
    pub fn disp(&self, index: usize) -> (f64, f64) {
        let ranges = self.columns_ranges.get().unwrap_or_else(|| {
            panic!(
                "{}.{} | Cache error: no columns ranges! index:{index}",
                self.dbg, "disp"
            )
        });
        
        assert!(
            ranges.len() > index,
            "Index {} is out of bounds for table with {} columns",
            index,
            ranges.len()
        );
        
        ranges[index]
    }
}
