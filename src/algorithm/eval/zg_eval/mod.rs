//! Evaluates multiple tasks in parallel
pub mod ctx;
pub mod eval;
///
/// Wrapper for Zg fix f64
pub struct Zg(pub f64);
impl Zg {
    pub fn empty() -> Self {
        Self(0.0)
    }
}
