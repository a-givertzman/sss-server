//! Структуры для ввода/вывода данных
// pub mod result;         // !!! to be moved to the Context
// pub mod check_result;
pub mod ship;
pub mod voyage;
pub mod ship_data;
pub mod data_array;
pub mod icing_stab;
pub mod icing_timber;
// pub mod loads;
// pub mod stability;
pub mod strength;
pub mod serde_parser;

pub mod math;
pub mod area;

pub use data_array::*;
pub use ship::*;
pub use voyage::*;
pub use ship_data::*;
pub mod stability;
// pub use strength::*;


// pub use stability::multipler_s::MultiplerSArray as MultiplerSArray;
// pub use stability::MultiplerX1Array as MultiplerX1Array;
// pub use stability::MultiplerX2Array as MultiplerX2Array;
// pub use stability::CoefficientKArray as CoefficientKArray;
// pub use stability::CoefficientKThetaArray as CoefficientKThetaArray;
// pub use stability::CenterDraughtShiftArray as CenterDraughtShiftArray;

pub type RadLongDataArray = DataArray<TrimVolumeData>;
pub type RadTransDataArray = DataArray<TrimVolumeData>;
pub type MetacentricHeightSubdivisionArray = DataArray<Pair>;
pub type MeanDraughtDataArray = DataArray<TrimVolumeData>;
pub type CenterWaterlineArray = DataArray<TrimVolumeData>;
pub type FloodingAngleDataArray = DataArray<TrimDraughtData>;
pub type EntryAngleDataArray = DataArray<TrimDraughtData>;
pub type WaterlineLengthArray = DataArray<Pair>;
pub type WaterlineBreadthArray = DataArray<TrimDraughtData>;
pub type WaterlineAreaArray = DataArray<TrimVolumeData>;
pub type VolumeShiftArray = DataArray<Pair>;
pub type BowAreaDataArray = DataArray<Pair>;


