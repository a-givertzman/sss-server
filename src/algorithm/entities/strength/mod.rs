//! Промежуточные структуры для serde_json для парсинга данных
//! для расчета прочности

pub(crate) mod mass;
pub(crate) mod icing;
pub(crate) mod wetting;
pub(crate) mod area;

pub use mass::*;
pub use icing::*;
pub use wetting::*;
pub use area::*;
