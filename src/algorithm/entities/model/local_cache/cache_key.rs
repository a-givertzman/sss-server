//!
//! Each implementors of [super::LocalCache] considered to be pointed by [CacheKey].
//!
//! It implements all required traits to be used as type of [IndexMap] key.
//! In addition, comparing to standard enums, this one can also be iterated over its variants.
//!
//! [IndexMap]: indexmap::IndexMap
//
use strum_macros::EnumIter;
///
/// Cache keys of [ShipModel] caches.
///
/// [ShipModel]: super::super::ShipModel
#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum CacheKey {
    ///
    /// Points to [FloatingPositionCache].
    ///
    /// [FloatingPositionCache]: super::floating_position_cache::FloatingPositionCache
    FloatingPostion,
}
