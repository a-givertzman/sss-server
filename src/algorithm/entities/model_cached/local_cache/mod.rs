//!
//! Defines a trait, which considered to be implemented
//! for all concrete cahces, used by [super::ShipModel].
//!
//! The triat provides an interface to
//! - calculate and store the dataset into configured file,
//! - reload the stored dataset for the current cache,
//! - calculate and get rows for given approximated values.
//
mod cache_key;
mod cache_conf;
mod file_io;
mod displacement_cache;
//mod bound_cache;
//mod compartment_cache;
mod windage_area_cache;
mod bounded_windage_area_cache;
mod local_cache;
mod shape;

pub(crate) use cache_key::*;
pub(crate) use cache_conf::*;
pub(crate) use file_io::*;
//pub(crate) use bound_cache::*;
pub(crate) use displacement_cache::*;
//pub(crate) use compartment_cache::*;
pub(crate) use windage_area_cache::*;
pub(crate) use bounded_windage_area_cache::*;
pub(crate) use local_cache::*;
pub(crate) use shape::*;
