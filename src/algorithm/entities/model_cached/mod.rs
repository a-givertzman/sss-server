//!
//! The representation of the ship in terms of its 3D elements.
//
mod local_cache;
mod model_cached_conf;
mod model_cached;
mod windage_area;
mod draught;
mod shape;
//mod test;

pub(crate) use local_cache::*;
pub use model_cached_conf::*;
pub(crate) use model_cached::*;
pub(crate) use windage_area::*;
pub(crate) use draught::*;
pub use shape::*;






