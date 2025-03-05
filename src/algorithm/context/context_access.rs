use sal_sync::services::entity::error::str_err::StrErr;

use super::{context::Context, ctx_result::CtxResult};
use crate::{
    algorithm::
        initial::initial_ctx::InitialCtx
    ,
};
///
/// Provides restricted write access to the [Context] members
pub trait ContextWrite<T> {
    fn write(self, value: T) -> CtxResult<Context, StrErr>;
}
///
/// Provides simple read access to the [Context] members
pub trait ContextRead<T> {
    fn read(&self) -> &T;
}
//
//
impl ContextWrite<InitialCtx> for Context {
    fn write(mut self, value: InitialCtx) -> CtxResult<Self, StrErr> {
        self.initial = value;
        CtxResult::Ok(self)
    }
}
impl ContextRead<InitialCtx> for Context {
    fn read(&self) -> &InitialCtx {
        &self.initial
    }
}
