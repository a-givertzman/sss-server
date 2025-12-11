use crate::CriterionDraughtCtx;
use crate::{
    algorithm::{eval::*, entities::data::stability::ship_type::ShipType},
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критериев посадки судна
pub struct CriterionDraughtEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl CriterionDraughtEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "CriterionDraughtEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for CriterionDraughtEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship = initial.ship.as_ref().unwrap();
                let ship_type = initial.ship_type.unwrap();
                let mut data = Vec::new();
                data.append(&mut ContextRead::<LoadLineCtx>::read(&ctx).data);
                //    out_data.append(&mut ContextRead::<TrimCtx>::read(&ctx).data);
                data.append(&mut ContextRead::<BowBoardCtx>::read(&ctx).data);
                data.append(&mut ContextRead::<ScrewCtx>::read(&ctx).data);
                if ship.freeboard_type == "B"
                    && !(ship_type == ShipType::Tanker
                        && ship_type == ShipType::OilTanker
                        && ship_type == ShipType::ChemicalTanker
                        && ship_type == ShipType::GasCarrier)
                {
                    data.push(ContextRead::<ReserveBuoyncyCtx>::read(&ctx).data);
                }
                let result = CriterionDraughtCtx { data };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for CriterionDraughtEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CriterionDraughtEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
