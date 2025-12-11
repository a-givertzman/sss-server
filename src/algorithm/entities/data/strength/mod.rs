//! Промежуточные структуры для serde_json для парсинга данных
//! для расчета прочности
pub mod computed_frame;
pub mod physical_frame;
pub mod frame_area;
pub mod horizontal_area;
pub mod vertical_area;

pub use computed_frame::*;
pub use physical_frame::*;
//pub use horizontal_area::*;
pub use vertical_area::*;
