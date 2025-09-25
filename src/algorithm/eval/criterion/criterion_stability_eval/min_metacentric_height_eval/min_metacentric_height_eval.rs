use super::min_metacentric_height_ctx::MinMetacentricHeightCtx;
use crate::{
    algorithm::{
        entities::data::{loads::UnitCargoType, ship_type::ShipType},
        eval::{CriterionData, CriterionID, LoadsCtx, MetacentricHeightCtx, zg_eval::Zg},
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия минимальной метацентрической высоты
pub struct MinMetacentricHeightEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl MinMetacentricHeightEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MinMetacentricHeightEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for MinMetacentricHeightEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let loads: LoadsCtx = ctx.read();
                let ship_type = initial.ship_type.unwrap();
                let have_grain = !loads.bulk.is_empty();
                let unit: Vec<_> = match initial.unit.as_ref() {
                    Some(data) => data
                        .into_iter()
                        .filter(|v| v.icing_area.is_some())
                        .collect(),
                    None => {
                        let error = error.err("Read unit error: no data!");
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::MinMetacentricHight,
                            "Ошибка расчета критерия минимальной метацентрической высоты: "
                                .to_owned()
                                + &error.to_string(),
                        );
                        let result = MinMetacentricHeightCtx { data: result };
                        return ctx.write(result);
                    }
                };
                let have_timber = unit.iter().any(|v| v.cargo_type == UnitCargoType::Timber);
                // Все суда
                let target = if have_grain {
                    0.3
                } else if ship_type == ShipType::RoRo {
                    0.2
                } else if have_timber {
                    0.1
                } else {
                    0.15
                };
                let result = MinMetacentricHeightCtx {
                    data: CriterionData::new_result(
                        CriterionID::MinMetacentricHight,
                        metacentric_height.h_trans_fix,
                        target,
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
impl std::fmt::Debug for MinMetacentricHeightEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinMetacentricHeightEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
