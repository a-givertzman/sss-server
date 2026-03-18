use crate::algorithm::eval::icing_timber_bound::ctx::IcingTimberBoundCtx;
use crate::algorithm::eval::icing_timber_bound::ctx::IcingTimberType;
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Ограничение горизонтальной площади обледенения палубного груза - леса
pub struct IcingTimberBoundEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl IcingTimberBoundEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingTimberBoundEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for IcingTimberBoundEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let icing_timber_stab = IcingTimberType::from_str(&voyage.icing_timber_type)
                    .map_err(|err| error.pass_with("icing_timber_stab", err))?;
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let length_loa = *ship_parameters
                    .get("L.O.A")
                    .ok_or(error.err("length_loa error: no data!"))?;
                let width = *ship_parameters
                    .get("MouldedBreadth")
                    .ok_or(error.err("width error: no data!"))?;
                let result = IcingTimberBoundCtx::new(width, length_loa, icing_timber_stab);
                log::info!(
                    "IcingTimberBound width:{:.3} length:{:.3} icing_timber_stab:{:?})",
                    result.width,
                    result.length,
                    result.icing_timber_stab,
                );                
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for IcingTimberBoundEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingTimberBoundEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
