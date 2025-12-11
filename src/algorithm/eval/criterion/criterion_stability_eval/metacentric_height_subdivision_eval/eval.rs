use crate::MetacentricHeightSubdivisionCtx;
use crate::algorithm::entities::{Curve, ICurve};
use crate::{
    algorithm::eval::{
        CriterionData, CriterionID, MetacentricHeightCtx, parameters::ParameterID, zg_eval::Zg,
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия метацентрической высоты
pub struct MetacentricHeightSubdivisionEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl MetacentricHeightSubdivisionEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MetacentricHeightSubdivisionEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for MetacentricHeightSubdivisionEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let data = initial.h_subdivision.as_ref().unwrap();
                let mean_draught = ctx.read_params(ParameterID::DraughtMean);
                let h_subdivision = Curve::new_linear(data)
                    .map_err(|err| error.pass_with("h_subdivision Curve::new_linear", err))?
                    .value(mean_draught)
                    .map_err(|err| error.pass_with("h_subdivision Curve::value", err))?;
                let result = MetacentricHeightSubdivisionCtx {
                    data: CriterionData::new_result(
                        CriterionID::MinMetacentricHeightSubdivIndex,
                        metacentric_height.h_trans_fix,
                        h_subdivision,
                    ),
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for MetacentricHeightSubdivisionEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetacentricHeightSubdivisionEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
