use sal_sync::services::entity::error::str_err::StrErr;

use super::{context::Context, ctx_result::CtxResult};
use crate::algorithm::{areas_strength::areas_strength_ctx::AreasStrengthCtx, icing_stab_eval::icing_stab_ctx::IcingStabCtx, initial::initial_ctx::InitialCtx};
///
/// Provides restricted write access to the [Context] members
pub trait ContextWrite<T> {
    fn write(self, value: T) -> CtxResult<Context, StrErr>;
}
///
/// Provides simple read access to the [Context] members
pub trait ContextReadRef<T> {
    fn read_ref(&self) -> &T;
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
    fn read_ref(&self) -> &InitialCtx {
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
//
impl ContextWrite<IcingStabCtx> for Context {
    fn write(mut self, value: IcingStabCtx) -> CtxResult<Self, StrErr> {
        self.icing_stab = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<IcingStabCtx> for Context {
    fn read(&self) -> IcingStabCtx {
        self.icing_stab.clone().unwrap()
    }
}

