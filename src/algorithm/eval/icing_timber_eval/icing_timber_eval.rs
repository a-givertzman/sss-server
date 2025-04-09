use sal_core::{dbg::Dbg, error::Error};
use crate::algorithm::context::context_access::*;
use crate::{
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use super::icing_timber_ctx::{IcingTimberCtx, IcingTimberType};

///
/// Ограничение горизонтальной площади обледенения палубного груза - леса
pub struct IcingTimberEval {
    dbg: Dbg,
    value: Option<IcingTimberCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl IcingTimberEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingTimberEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for IcingTimberEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let voyage = match initial.voyage.clone() {
                    Some(data) => data,
                    None => {
                        return CtxResult::Err(error.err("Read voyage error: no data!"))
                    }
                };
                let icing_timber_stab = match IcingTimberType::from_str(&voyage.icing_timber_type) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(error.pass_with("Read icing_timber_stab error", err))
                    }
                };
                let ship_parameters = match initial.ship_parameters.as_ref() {
                    Some(data) => data,
                    None => {
                        return CtxResult::Err(error.err("Read voyage error: no data!"))
                    }
                };
                let length_loa = match ship_parameters.get("L.O.A") {
                    Some(data) => *data,
                    None => {
                        return CtxResult::Err(error.err("Read length_loa error: no data!"))
                    }
                };
                let width = match ship_parameters.get("MouldedBreadth") {
                    Some(data) => *data,
                    None => {
                        return CtxResult::Err(error.err("Read width error: no data!"))
                    }
                };                 
                let result = IcingTimberCtx::new(width, length_loa, icing_timber_stab);
                self.value = Some(result.clone());
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for IcingTimberEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingTimberEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
