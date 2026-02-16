mod common;
/// TODO: To be deleted !
mod examples;
mod query_id;
mod query;

pub(crate) use common::*;
/// TODO: To be deleted !
pub(crate) use examples::*;
pub(crate) use query_id::*;
pub(crate) use query::*;

/// Configuration for the binary encoding / decoding with default parameters
/// - Little endian
/// - Variable int encoding
pub const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();
