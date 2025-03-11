use crate::data::structs::LoadLineDataArray;

#[allow(dead_code)]
pub(crate) fn load_line() -> LoadLineDataArray {
    LoadLineDataArray::from(vec![
        (102, 0., -6.7, 4.6),
        (101, 0., 6.7, 4.6),
        (110, 0.425, -6.7, 4.708),
        (109, 0.425, 6.7, 4.708),
    ])
}