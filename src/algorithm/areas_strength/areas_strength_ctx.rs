use crate::algorithm::entities::{area::HAreaStrength, data::strength::VerticalArea};
///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct AreasStrengthCtx {
    /// разбиение на шпации - фреймы
    pub areas: (Vec<VerticalArea>, Vec<HAreaStrength>),
}
//
//
// impl Default for AreasStrengthCtx {
//     ///
//     /// Struct constructor
//     /// - 'storage_initial_data' - [Storage] instance, where store initial data
//     fn default() -> Self {
//         Self {
//             bounds: None,
//         }
//     }
// }
