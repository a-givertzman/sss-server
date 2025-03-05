use sal_sync::services::entity::error::str_err::StrErr;

use super::{context::Context, ctx_result::CtxResult};
use crate::algorithm::{areas_strength::areas_strength_ctx::AreasStrengthCtx, 
        initial::initial_ctx::InitialCtx};
///
/// Provides restricted write access to the [Context] members
pub trait ContextWrite<T> {
    fn write(self, value: T) -> CtxResult<Context, StrErr>;
}
///
/// Provides simple read access to the [Context] members
pub trait ContextReadRef<T> {
    fn read(&self) -> &T;
}
///
/// Provides simple read access to the [Context] members
pub trait ContextRead<T> {
    fn read(&self) -> T;
}
//
//
impl ContextWrite<InitialCtx> for Context {
    fn write(mut self, value: InitialCtx) -> CtxResult<Self, StrErr> {
        self.initial = value;
        CtxResult::Ok(self)
    }
}
impl ContextReadRef<InitialCtx> for Context {
    fn read(&self) -> &InitialCtx {
        &self.initial
    }
}
//
//
//
//
impl ContextWrite<AreasStrengthCtx> for Context {
    fn write(mut self, value: AreasStrengthCtx) -> CtxResult<Self, StrErr> {
        self.areas_strength = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<AreasStrengthCtx> for Context {
    fn read(&self) -> AreasStrengthCtx {
        self.areas_strength.clone().unwrap()
    }
}
