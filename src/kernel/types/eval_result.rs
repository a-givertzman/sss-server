use sal_sync::services::entity::error::str_err::StrErr;

use crate::algorithm::context::{context::Context, ctx_result::CtxResult};
///
/// Result returned from Calculation steps
// pub type EvalResult = (Switch, CtxResult<Context, StrErr>);
pub type EvalResult = CtxResult<Context, StrErr>;