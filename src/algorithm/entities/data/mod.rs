//! Структуры для ввода/вывода данных
//pub mod result;
//pub mod check_result;
pub mod ship;
pub mod voyage;
pub mod ship_data;
pub mod data_array;
pub mod loads;
pub mod stability;
pub mod strength;
pub mod serde_parser;

pub use data_array::*;
pub use ship::*;
pub use voyage::*;
pub use ship_data::*;

pub use stability::multipler_s::MultiplerSArray as MultiplerSArray;
pub use stability::MultiplerX1Array as MultiplerX1Array;
pub use stability::MultiplerX2Array as MultiplerX2Array;
pub use stability::CoefficientKArray as CoefficientKArray;
pub use stability::CoefficientKThetaArray as CoefficientKThetaArray;

pub type MetacentricHeightSubdivisionArray = DataArray<Pair>;


