use sal_core::error::Error;
use super::{context::Context, ctx_result::CtxResult};
use crate::algorithm::{entities::parameters::{IParameters, ParameterID, Parameters}, eval::*, initial::initial_ctx::InitialCtx};
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
///
/// Provides restricted write access to the [Context].[Parameters] members
pub trait ContextParamsWrite<T> {
    fn write(self, key: ParameterID, value: T) -> CtxResult<Context, Error>;
}
///
/// Provides simple read access to the [Context].[Parameters] members
pub trait ContextParamsRead<T> {
    fn read(&self, key: ParameterID) -> T;
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
impl ContextWrite<Parameters> for Context {
    fn write(mut self, value: Parameters) -> CtxResult<Self, Error> {
        self.parameters = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextReadRef<Parameters> for Context {
    fn read_ref(&self) -> &Parameters {
        self.parameters
            .as_ref()
            .unwrap()
    }
}
impl ContextParamsWrite<f64> for Context {
    fn write(mut self, id: ParameterID, value: f64) -> CtxResult<Self, Error> {
        match &mut self.parameters {
            Some(params) => {
                params.add(id, value);
            }
            None => panic!("Context.write | Parameters - is not initialised yet, id: {:?}", id)
        };
        CtxResult::Ok(self)
    }
}
impl ContextParamsRead<f64> for Context {
    fn read(&self, id: ParameterID) -> f64 {
        let params: &Parameters  = self.read_ref();
        params.get(id).expect(&format!("Context.read | Id '{:?}' - is not found", id))
    }
}







