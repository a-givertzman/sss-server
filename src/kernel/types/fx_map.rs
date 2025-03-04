use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
///
/// Lightweght faster HashMap
pub type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher>>;
///
/// Lightweght faster IndexMap
pub type FxIndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<FxHasher>>;
///
/// Lightweght faster DashMap
pub type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;
