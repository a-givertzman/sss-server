//! Evaluates multiple tasks in parallel
pub mod ctx;
pub mod eval;
///
/// Wrapper for Zg fix f64
#[derive(Debug, Clone)]
pub struct Zg(pub Option<f64>);

impl Zg {
    pub fn empty() -> Self {
        Self(None)
    }
}
