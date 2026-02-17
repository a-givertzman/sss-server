use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::eval::seakeeping::period_excitement::period_excitement_ctx::PeriodExcitementCtx;
use crate::prelude::{ContextWrite, InitialCtx};
use crate::kernel::{
        Eval, 
        types::eval_result::EvalResult
    };
use sal_core::{
    dbg::Dbg, 
    error::Error
};
///
/// Расчет [периода волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct PeriodExcitementEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl PeriodExcitementEval {
    ///
    /// Новый экземпляр [PeriodExcitementEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,) -> Self {
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
                let initial_data = ContextReadRef::<InitialCtx>::read_ref(&ctx).clone();
                if initial_data.period_excitement.is_none() {
                    let length_wave = initial_data.wave_length.unwrap();
                    ctx.write(
                        PeriodExcitementCtx {
                            period_excitement: (length_wave / 1.56).sqrt(),
                        }
                    )
                } else {
                    ctx.write(
                        PeriodExcitementCtx {
                            period_excitement: initial_data.period_excitement.unwrap().period_excitement.clone(),
                        }
                    )
                }
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for PeriodExcitementEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeriodExcitementEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
