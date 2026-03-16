//! Коллекция примитивов для математических операций
pub mod vec;
mod position_2d;
pub mod position;
pub mod moment;
pub mod curve;
pub mod bound;
pub mod pos_shift;
pub mod liquid;
pub mod bounds;
pub mod build_wall;
pub mod points_manipulation;
pub mod is_generate;
#[allow(unused)]
pub use vec::integral_sum::IntegralSum as IntegralSum;
#[allow(unused)]
pub use vec::integral_cotes::IntegralCotes as IntegralCotes;
#[allow(unused)]
pub use vec::integral::Integral as Integral;
pub use vec::*;
pub use position_2d::*;
pub use position::*;
pub mod resample_line;
pub use moment::*;
pub use curve::*;
pub use bound::*;
//pub use pos_shift::*;
pub use liquid::*;
pub use bounds::*;