//! Коллекция примитивов для математических операций
pub mod vec;
pub mod position;
pub mod moment;
pub mod curve;
pub mod bound;
pub mod liquid;
pub mod bounds;
mod draught;

#[allow(unused)]
pub use vec::integral_cotes::IntegralCotes as IntegralCotes;
#[allow(unused)]
pub use vec::integral::Integral as Integral;
pub use vec::*;
pub use position::*;
pub use moment::*;
pub use curve::*;
pub use bound::*;
//pub use pos_shift::*;
pub use liquid::*;
pub use bounds::*;
pub use draught::*;