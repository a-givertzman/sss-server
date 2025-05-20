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
mod floating_position_cache;
mod local_cache;

pub(crate) use cache_key::*;
pub(crate) use floating_position_cache::*;
pub(crate) use local_cache::*;
