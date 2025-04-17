use sal_core::error::Error;
use super::{context::Context, ctx_result::CtxResult};
use crate::algorithm::{eval::*, initial::initial_ctx::InitialCtx};
///
/// Provides restricted write access to the [Context] members
pub trait ContextWrite<T> {
    fn write(self, value: T) -> CtxResult<Context, Error>;
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
    fn write(mut self, value: InitialCtx) -> CtxResult<Self, Error> {
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
impl ContextWrite<Parameters> for Context {
    fn write(mut self, value: Parameters) -> CtxResult<Self, Error> {
        self.parameters = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<Parameters> for Context {
    fn read(&self) -> Parameters {
        self.parameters.clone().unwrap()
    }
}
//
impl ContextWrite<StrengthAreaCtx> for Context {
    fn write(mut self, value: StrengthAreaCtx) -> CtxResult<Self, Error> {
        self.strength_area = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<StrengthAreaCtx> for Context {
    fn read(&self) -> StrengthAreaCtx {
        self.strength_area.clone().unwrap()
    }
}
//
impl ContextWrite<IcingStabCtx> for Context {
    fn write(mut self, value: IcingStabCtx) -> CtxResult<Self, Error> {
        self.icing_stab = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<IcingStabCtx> for Context {
    fn read(&self) -> IcingStabCtx {
        self.icing_stab.clone().unwrap()
    }
}
//
impl ContextWrite<IcingCtx> for Context {
    fn write(mut self, value: IcingCtx) -> CtxResult<Self, Error> {
        self.icing = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<IcingCtx> for Context {
    fn read(&self) -> IcingCtx {
        self.icing.clone().unwrap()
    }
}
//
impl ContextWrite<WettingCtx> for Context {
    fn write(mut self, value: WettingCtx) -> CtxResult<Self, Error> {
        self.wetting = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<WettingCtx> for Context {
    fn read(&self) -> WettingCtx {
        self.wetting.clone().unwrap()
    }
}
//
impl ContextWrite<LoadsCtx> for Context {
    fn write(mut self, value: LoadsCtx) -> CtxResult<Self, Error> {
        self.loads = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<LoadsCtx> for Context {
    fn read(&self) -> LoadsCtx {
        self.loads.clone().unwrap()
    }
}
//
impl ContextWrite<IcingTimberCtx> for Context {
    fn write(mut self, value: IcingTimberCtx) -> CtxResult<Self, Error> {
        self.icing_timber = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<IcingTimberCtx> for Context {
    fn read(&self) -> IcingTimberCtx {
        self.icing_timber.clone().unwrap()
    }
}
//
impl ContextWrite<BalanceCtx> for Context {
    fn write(mut self, value: BalanceCtx) -> CtxResult<Self, Error> {
        self.balance = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<BalanceCtx> for Context {
    fn read(&self) -> BalanceCtx {
        self.balance.clone().unwrap()
    }
}
//
impl ContextWrite<StabilityAreaCtx> for Context {
    fn write(mut self, value: StabilityAreaCtx) -> CtxResult<Self, Error> {
        self.stability_area = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<StabilityAreaCtx> for Context {
    fn read(&self) -> StabilityAreaCtx {
        self.stability_area.clone().unwrap()
    }
}
//
impl ContextWrite<MetacentricHeightCtx> for Context {
    fn write(mut self, value: MetacentricHeightCtx) -> CtxResult<Self, Error> {
        self.metacentric_height = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<MetacentricHeightCtx> for Context {
    fn read(&self) -> MetacentricHeightCtx {
        self.metacentric_height.clone().unwrap()
    }
}
//
impl ContextWrite<LeverDiagramCtx> for Context {
    fn write(mut self, value: LeverDiagramCtx) -> CtxResult<Self, Error> {
        self.lever_diagram = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<LeverDiagramCtx> for Context {
    fn read(&self) -> LeverDiagramCtx {
        self.lever_diagram.clone().unwrap()
    }
}
//
impl ContextWrite<WindCtx> for Context {
    fn write(mut self, value: WindCtx) -> CtxResult<Self, Error> {
        self.wind = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<WindCtx> for Context {
    fn read(&self) -> WindCtx {
        self.wind.clone().unwrap()
    }
}
//
impl ContextWrite<WindageCtx> for Context {
    fn write(mut self, value: WindageCtx) -> CtxResult<Self, Error> {
        self.windage = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<WindageCtx> for Context {
    fn read(&self) -> WindageCtx {
        self.windage.clone().unwrap()
    }
}
//
impl ContextWrite<RollingPeriodCtx> for Context {
    fn write(mut self, value: RollingPeriodCtx) -> CtxResult<Self, Error> {
        self.roll_period = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<RollingPeriodCtx> for Context {
    fn read(&self) -> RollingPeriodCtx {
        self.roll_period.clone().unwrap()
    }
}








