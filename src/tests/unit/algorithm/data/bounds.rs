use crate::algorithm::entities::Bounds;

#[allow(dead_code)]
pub(crate) fn bounds() -> Bounds {
    Bounds::from_min_max(stern_x, bow_x, n_parts).unwrap()
}
