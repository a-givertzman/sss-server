use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::Draught;
use crate::algorithm::eval::criterion::*;
use crate::prelude::InitialCtx;
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия заглубления винта
pub struct ScrewEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ScrewEval {
    //
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ScrewEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for ScrewEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let data = initial.screw.as_ref().unwrap();
                let mut result = Vec::new();
                let draught = Draught::new(&self.dbg, &ctx).map_err(|err| error.pass(err))?;
                for v in data.iter() {
                    let z_fix = v.pos.z() - draught.value(&v.pos);
                    let percent = (1. - z_fix / v.d).clamp(0., 2.) * 50.;
                    log::info!(
                        "Criterion ScrewImmersion point:{} z_fix:{:.3} percent:{:.3}",
                        v.pos.print(),
                        z_fix,
                        percent
                    );
                    result.push(if v.pos.y() < -1. {
                        CriterionData::new_result(CriterionID::ScrewImmersionPS, percent, 100.)
                    } else if v.pos.y() > 1. {
                        CriterionData::new_result(CriterionID::ScrewImmersionSB, percent, 100.)
                    } else {
                        CriterionData::new_result(CriterionID::ScrewImmersionCL, percent, 100.)
                    });
                }
                let result = ScrewCtx { data: result };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ScrewEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrewEval").field("dbg", &self.dbg).finish()
    }
}
