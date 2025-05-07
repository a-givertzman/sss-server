use sal_core::{dbg::Dbg, error::Error};
use super::bound::Bound;
use super::OwnedSet;
use std::{cmp::Ordering, ops::Deref};
///
/// Analyzed dataset, column of a [super::Table] instance.
///
/// A dataset is _analyzed_ if all its inflection points are defined.
#[derive(Clone, Debug)]
pub(super) struct Column<T> {
    inflections: OwnedSet<usize>,
    data: OwnedSet<T>,
    Dbg: Dbg,
}
//
//
impl<T: PartialOrd> Column<T> {
    ///
    /// Returns an analyzed instance (see [Column] for details).
    ///
    /// # Panics
    /// Panic occurs if `values` contains a non-comparable value (e. g. _NaN_).
    pub(super) fn new<S>(Dbg: Dbg, values: S) -> Self
    where
        S: Into<OwnedSet<T>> + Deref<Target = [T]>,
    {
        Self {
            inflections: Self::get_inflections(&Dbg, &values),
            data: values.into(),
            Dbg,
        }
    }
    ///
    /// Returns inflection point IDs based on given values.
    ///
    /// # Panics
    /// Panic occurs if `values` contains a non-comparable value (e. g. _NaN_).
    fn get_inflections(Dbg: &Dbg, values: &[T]) -> OwnedSet<usize> {
        use Ordering::*;
        //
        if values.is_empty() {
            return OwnedSet::from([]);
        }
        let callee = "get_inflections";
        // inflection points
        let mut flex = vec![];
        // keep last flex direction
        let mut prev_dir = None;
        // iterate over [.., l_val, m_val, r_val, ..] values
        // with middle index:         ^ m_id
        for (win, m_id) in values.windows(3).zip(1..) {
            let l_val = &win[0];
            let m_val = &win[1];
            let r_val = &win[2];
            match (l_val.partial_cmp(m_val), m_val.partial_cmp(r_val)) {
                (Some(l_vs_m), Some(m_vs_r)) => match (l_vs_m, m_vs_r) {
                    // flex (p)oint: m_id __. . or   .
                    //                     . \    . .__ m_id
                    //                        p     ^p
                    (cur @ Less, Equal)
                    | (Equal, cur @ Less)
                    | (cur @ Greater, Equal)
                    | (Equal, cur @ Greater) => match prev_dir.as_mut() {
                        Some(prev) => {
                            if cur != *prev {
                                flex.push(m_id);
                                prev_dir = Some(cur);
                            }
                        }
                        None => prev_dir = Some(cur),
                    },
                    // local extremum
                    (Greater, cur @ Less) | (Less, cur @ Greater) => {
                        flex.push(m_id);
                        prev_dir = Some(cur);
                    }
                    _ => {}
                },
                _ => panic!(
                    "{}.{} | Non-comparable value at position {}, {} or {}",
                    Dbg,
                    callee,
                    m_id - 1,
                    m_id,
                    m_id + 1
                ),
            }
        }
        // Include the first, ..
        let mut ids = vec![0];
        if !flex.is_empty() {
            // .. middle, ..
            ids.extend(flex);
        }
        // ... and last indexes.
        ids.push(values.len() - 1);
        // remove duplicates
        ids.dedup();
        ids.into()
    }
    ///
    /// Returns bounds of given value within internal dataset.
    ///
    /// # Panics
    /// Panic occurs if `val` is a non-comparable value (e. g. _NaN_).
    pub(super) fn get_bounds(&self, val: &T) -> Vec<Bound>
    where
        T: std::fmt::Display,
    {
        use Ordering::*;
        //
        let callee = "get_bounds";
        // walk through all middle values
        let iter = self
            .inflections
            .windows(2)
            .filter(|win| {
                let first = &self.data[win[0]];
                let last = &self.data[win[1]];
                match (first.partial_cmp(val), val.partial_cmp(last)) {
                    (Some(first_vs_val), Some(val_vs_last)) => matches!(
                        (first_vs_val, val_vs_last),
                        (Less | Equal, Less | Equal) | (Greater | Equal, Greater | Equal)
                    ),
                    _ => panic!(
                        "{}.{} | `val`={} is a non-comparable value",
                        self.Dbg, callee, val
                    ),
                }
            })
            .flat_map(|win| {
                Self::get_bounds_of_monotonic(&self.data[win[0]..=win[1]], val, win[0])
            });
        let mut bounds = Vec::from_iter(iter);
        bounds.dedup();
        bounds
    }
    ///
    /// Returns bounds of given element (`val`) placing in between elemnts of `vals`,
    /// where `offset` represents the actual start index of `vals`. For internal use.
    ///
    /// # Note
    /// - If `vals` is not monotonic, the output is _meaningless_.
    /// - This is an unsafe method: Caller _must garantee_ that `vals` and `val` are comparable values.
    fn get_bounds_of_monotonic(vals: &[T], val: &T, offset: usize) -> Vec<Bound> {
        let mut bounds = vec![];
        let dir = vals[0].partial_cmp(&vals[vals.len() - 1]);
        let insert_id = vals.partition_point(|data_val| data_val.partial_cmp(val) == dir);
        if insert_id == 0 {
            bounds.push(Bound::Single(offset));
        } else if insert_id == vals.len() {
            bounds.push(Bound::Single(vals.len() - 1 + offset));
        } else if *val == vals[insert_id] {
            bounds.push(Bound::Single(insert_id + offset));
            match dir {
                Some(Ordering::Less | Ordering::Greater) => {
                    (1..)
                        .take_while(|i| {
                            vals.get(insert_id + i)
                                .is_some_and(|insert_val| insert_val == val)
                        })
                        .for_each(|i| {
                            bounds.push(Bound::Single(insert_id + i + offset));
                        });
                }
                _ => unreachable!(),
            }
        } else {
            let start = insert_id - 1 + offset;
            let end = insert_id + offset;
            bounds.push(Bound::Range(start, end));
        }
        bounds
    }
}
//
//
impl<T> Deref for Column<T> {
    type Target = [T];
    //
    //
    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}
