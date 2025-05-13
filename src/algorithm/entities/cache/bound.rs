//
use std::ops::BitAnd;
///
/// Represents bound(s) of element within the collection.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Bound {
    ///
    /// No bound.
    None,
    ///
    /// Index of the match.
    Single(usize),
    ///
    /// Indexes of either two nearest neighbors,
    /// or the start and end of exact match range.
    Range(usize, usize),
}
//
//
impl BitAnd for Bound {
    type Output = Self;
    //
    //
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bound::Single(this), Bound::Single(other)) if this == other => Bound::Single(this),
            (Bound::Single(id), Bound::Range(start, end))
            | (Bound::Range(start, end), Bound::Single(id))
                if (start..=end).contains(&id) =>
            {
                Bound::Single(id)
            }
            (Bound::Range(mut start1, mut end1), Bound::Range(mut start2, mut end2)) => {
                if start2 < start1 {
                    std::mem::swap(&mut start1, &mut start2);
                    std::mem::swap(&mut end1, &mut end2);
                }
                if end1 < start2 {
                    return Bound::None;
                }
                let start = start1.max(start2);
                let end = end1.min(end2);
                match start == end {
                    true => Bound::Single(start),
                    _ => Bound::Range(start, end),
                }
            }
            _ => Bound::None,
        }
    }
}
