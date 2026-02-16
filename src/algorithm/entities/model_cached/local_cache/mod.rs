//!
//! Defines a trait, which considered to be implemented
//! for all concrete cahces, used by [super::ShipModel].
//!
//! The triat provides an interface to
//! - calculate and store the dataset into configured file,
//! - reload the stored dataset for the current cache,
//! - calculate and get rows for given approximated values.
//
mod file_io;
mod local_cache;
mod displacement_bound_cache;
mod compartment_bound_cache;
mod displacement_cache;
mod compartment_cache;
mod damaged_compartment_cache;
mod windage_cache;
mod bow_area_cache;
mod hold_compartment_cache;
mod hold_compartment_bound_cache;

pub(crate) use file_io::*;
pub(crate) use local_cache::*;
pub(crate) use displacement_bound_cache::*;
pub(crate) use compartment_bound_cache::*;
pub(crate) use displacement_cache::*;
pub(crate) use compartment_cache::*;
pub(crate) use damaged_compartment_cache::*;
pub(crate) use windage_cache::*;
pub(crate) use bow_area_cache::*;
pub(crate) use hold_compartment_cache::*;
pub(crate) use hold_compartment_bound_cache::*;

