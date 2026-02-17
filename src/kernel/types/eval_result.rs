use sal_core::error::Error;
use crate::algorithm::context::{context::Context};
///
/// Result returned from Calculation steps
pub type EvalResult = Result<Context, Error>;