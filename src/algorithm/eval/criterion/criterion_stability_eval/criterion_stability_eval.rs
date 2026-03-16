use super::criterion_stability_ctx::CriterionStabilityCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::data::{loads::UnitCargoType, ship_type::ShipType, NavigationArea},
        eval::{zg_eval::Zg, *},
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критериев проверки остойчивости судна
pub struct CriterionStabilityEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl CriterionStabilityEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "CriterionStabilityEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for CriterionStabilityEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let navigation_area = initial.navigation_area.unwrap();
                let ship_type = initial.ship_type.unwrap();
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let h_trans_fix = metacentric_height.h_trans_fix;
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let breadth = *ship_parameters
                    .get("MouldedBreadth")
                    .ok_or(error.err("breadth error: no data!"))?;
                let moulded_depth = *ship_parameters
                    .get("Moulded depth")
                    .ok_or(error.err("moulded_depth error: no data!"))?;
                let have_container = initial
                    .unit
                    .as_ref()
                    .ok_or(error.err("initial.unit no data"))?
                    .into_iter()
                    .any(|v| v.cargo_type == UnitCargoType::Container);
                let loads: LoadsCtx = ctx.read();
                let have_grain = !loads.bulk.is_empty();
                let mut data = Vec::new();
                if navigation_area != NavigationArea::R3Rsn {
                    data.push(ContextRead::<WheatherCtx>::read(&ctx).data);
                }
                if navigation_area != NavigationArea::R3Rsn {
                    data.push(ContextRead::<StaticAngleCtx>::read(&ctx).data);
                }
                data.append(&mut ContextRead::<DSOAreaCtx>::read(&ctx).data);
                let icing_stab: IcingStabCtx = ctx.read();
                let have_icing = icing_stab.is_some;
                match (
                    have_icing,
                    navigation_area == NavigationArea::Unrestricted,
                    ship_type == ShipType::TimberCarrier,
                ) {
                    // обледенение, не лесовоз, неограниченный район
                    (true, true, false) => {
                        data.push(ContextRead::<DSOMaxCtx>::read(&ctx).data) // 2.2.1.2    
                    }
                    // обледенение, не лесовоз, ограниченный район
                    (true, false, false) => {
                        data.push(ContextRead::<DSOIcingMaxCtx>::read(&ctx).data) // 2.4.9
                    }
                    // обледенение, лесовоз, неограниченный район
                    (true, true, true) => {
                        data.push(ContextRead::<DSOTimberMaxCtx>::read(&ctx).data) // 3.3.5  
                    }
                    (true, false, true) => {
                        // обледенение, лесовоз, ограниченный район
                        data.push(ContextRead::<DSOIcingMaxCtx>::read(&ctx).data); // 2.4.9
                        data.push(ContextRead::<DSOTimberMaxCtx>::read(&ctx).data); // 3.3.5   
                    }
                    // без обледенения, лесовоз
                    (false, _, true) => {
                        data.push(ContextRead::<DSOTimberMaxCtx>::read(&ctx).data) // 3.3.5
                    }
                    // без обледенения, все суда
                    (false, _, _) => {
                        data.push(ContextRead::<DSOMaxCtx>::read(&ctx).data) // 2.2.1.2   
                    }
                }
                data.append(&mut ContextRead::<DSOAngleMaxCtx>::read(&ctx).data);
                //if self.have_cargo {
                data.push(ContextRead::<MinMetacentricHeightCtx>::read(&ctx).data);
                //}
                if navigation_area == NavigationArea::R2Rsn
                    || navigation_area == NavigationArea::R2Rsn45
                    || h_trans_fix.sqrt() / breadth > 0.08
                    || breadth / moulded_depth > 2.5
                {
                    data.push(ContextRead::<AccelerationCtx>::read(&ctx).data);
                }
                if have_container {
                    data.push(ContextRead::<CirculationCtx>::read(&ctx).data);
                }
                if have_grain {
                    data.append(&mut ContextRead::<GrainCtx>::read(&ctx).data);
                }
                data.push(ContextRead::<MetacentricHeightSubdivisionCtx>::read(&ctx).data);
                // TODO        HeelMaximumLC, HeelFirstMaximumLC
                // data.push(CriterionData::new_result(CriterionID::HeelMaximumLC , self.lever_diagram.max_angles(), 1.);
            //    info!("Criterion stability end");
                let result = CriterionStabilityCtx { data };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for CriterionStabilityEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CriterionStabilityEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
