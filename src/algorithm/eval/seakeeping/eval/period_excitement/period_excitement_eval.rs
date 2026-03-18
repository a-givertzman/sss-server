use sal_core::{dbg::Dbg, error::Error};

use crate::{
    algorithm::eval::seakeeping::eval::period_excitement::period_excitement_ctx::PeriodExcitementCtx,
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
};
///
/// Расчет [периода волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct PeriodExcitementEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync + 'static>,
}
//
//
impl PeriodExcitementEval {
    ///
    /// Новый экземпляр [PeriodExcitementEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "PeriodExcitementEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for PeriodExcitementEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial = ContextReadRef::<InitialCtx>::read_ref(&ctx);
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let wave_length = voyage.wave_length;
                ctx.write(
                    PeriodExcitementCtx {
                        period_excitement: (wave_length / 1.56).sqrt(),
                    }
                )
            }
            Err(err) => Err(err),
        }
    }
}
