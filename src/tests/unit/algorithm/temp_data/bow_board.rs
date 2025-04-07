use crate::data::structs::BowBoardDataArray;

#[allow(dead_code)]
pub(crate) fn bow_board() -> BowBoardDataArray {
    BowBoardDataArray::from(vec![
        (143, 59.193, 1.251, 9.696),
        (144, 59.193, -1.251, 9.696),
    ])
}