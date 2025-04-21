use sal_core::error::Error;
use crate::algorithm::context::{context::Context, ctx_result::CtxResult};
///
/// Result returned from Calculation steps
pub type EvalResult = CtxResult<Context, Error>;