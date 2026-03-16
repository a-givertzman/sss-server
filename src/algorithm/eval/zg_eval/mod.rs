//! Evaluates multiple tasks in parallel
pub mod zg_ctx;
pub mod zg_eval;
///
/// Wrapper for Zg fix f64
pub struct Zg(pub f64);
impl Zg {
    pub fn empty() -> Self {
        Self(0.0)
    }
}
