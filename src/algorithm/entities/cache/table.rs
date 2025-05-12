use sal_core::{dbg::Dbg, error::Error};
use super::{bound::Bound, column::Column, OwnedSet};
///
/// Set of [Column]s.
pub(super) struct Table<T> {
    dbg: Dbg,
    columns: OwnedSet<Column<T>>,
}
//
//
impl<T> Table<T> {
    ///
    /// Creates a new instance.
    pub(super) fn new(parent: &Dbg, cols: impl Into<OwnedSet<Column<T>>>) -> Self {
        let dbg = Dbg::new(parent, "Table");
        let columns = cols.into();
        Self { dbg, columns }
    }
}
//
//
impl Table<f64> {
    ///
    /// Returns approximated values from table.
    ///
    /// This is a safe method in terms if bounds: If `approx_vals` has more elements than [Table] row provides,
    /// this method returns [None]. In contrast, the empty vector returns if no value found.
    ///
    /// # Panics
    /// Panic occurs if `approx_vals` contains a non-comparable value (e. g. _NaN_).
    pub(super) fn get(&self, approx_vals: &[Option<f64>]) -> Option<Vec<Vec<f64>>> {
        (approx_vals.len() <= self.columns.len()).then(|| self.get_unchecked(approx_vals))
    }
    ///
    /// Returns approximated values from table.
    ///
    /// Note that this is an unsafe version for internal use.
    /// Caller must garantee that `approx_vals.len()` is less or equal to `self.columns.len()`.
    ///
    /// # Panics
    /// This method panics if at least one of the statements is true:
    /// - `approx_vals.len()` is greter than `self.columns.len()`,
    /// - `approx_vals` contains a non-comparable value (e. g. _NaN_) (see [Column::get_bounds]).
    fn get_unchecked(&self, approx_vals: &[Option<f64>]) -> Vec<Vec<f64>> {
        let callee = "get_unchecked";
        assert!(
            self.columns.len() >= approx_vals.len(),
            "{}.{} | columns.len={} < approx_vals.len={}",
            self.dbg,
            callee,
            self.columns.len(),
            approx_vals.len()
        );
        let bounds = {
            let mut val_bounds = vec![];
            for (id, val) in approx_vals
                .iter()
                .enumerate()
                .filter_map(|(id, val)| val.as_ref().map(|val| (id, val)))
            {
                let bounds = self.columns[id].get_bounds(val);
                val_bounds.push(bounds);
            }
            log::debug!(
                "{}.{} | Filtered bounds: {:?}",
                self.dbg,
                callee,
                val_bounds
            );
            loop {
                match (val_bounds.pop(), val_bounds.last_mut()) {
                    (None, _) => return vec![],
                    (Some(bounds), Some(last_bounds)) => {
                        //
                        // NOTE: switch between last_bounds and bounds may increase perf
                        *last_bounds = last_bounds
                            .iter()
                            .copied()
                            .flat_map(|last_bound| {
                                bounds
                                    .iter()
                                    .copied()
                                    .filter_map(|bound| match last_bound & bound {
                                        Bound::None => None,
                                        bound => Some(bound),
                                    })
                                    .collect::<Vec<Bound>>()
                            })
                            .collect();
                    }
                    (Some(mut last), None) => {
                        last.dedup();
                        break last;
                    }
                }
            }
        };
        log::debug!("{}.{} | Merged bounds: {:?}", self.dbg, callee, bounds);
        bounds
            .into_iter()
            .flat_map(|bound| match bound {
                Bound::None => None,
                Bound::Single(row_id) => {
                    let mut vals = Vec::with_capacity(self.columns.len());
                    for col in self.columns.iter() {
                        let val = col[row_id];
                        vals.push(val);
                    }
                    Some(vals)
                }
                Bound::Range(start, end) => {
                    let mut vals = Vec::with_capacity(self.columns.len());
                    let len = end - start + 1;
                    for (col_id, col) in self.columns.iter().enumerate() {
                        let sum = (start..=end).map(|row_id| col[row_id]).sum::<f64>();
                        let val = sum / len as f64;
                        vals.push(val);
                        log::trace!(
                            "{}.{} | Interpolation: col_id={} from row_id={} to row_id={} with result={}",
                            self.dbg, callee, col_id, start, end, val
                        );
                    }
                    Some(vals)
                }
            })
            .collect()
    }
}
