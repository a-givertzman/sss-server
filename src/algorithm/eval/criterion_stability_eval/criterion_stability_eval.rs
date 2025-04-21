use super::criterion_stability_ctx::CriterionStabilityCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef}, eval::MetacentricHeightCtx,
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::{IModelLink, ModelLink}, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет периода качки судна 
pub struct CriterionStabilityEval {
    dbg: Dbg,
    value: Option<CriterionStabilityCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl CriterionStabilityEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "CriterionStabilityEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for CriterionStabilityEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                log::info!("Criterion begin");
                let initial: &InitialCtx = ctx.read_ref();                                
                let mut result = Vec::new();
                if self.navigation_area != NavigationArea::R3Rsn {
                    result.push(self.weather());
                }
                if self.navigation_area != NavigationArea::R3Rsn {
                    result.push(self.static_angle());
                }
                result.append(&mut self.dso());
                match (self.have_icing, self.navigation_area == NavigationArea::Unrestricted, self.ship_type == ShipType::TimberCarrier) {
                    // обледенение, не лесовоз, неограниченный район
                    (true, true, false) => result.push(self.dso_lever()), // 2.2.1.2            
                    // обледенение, не лесовоз, ограниченный район
                    (true, false, false) => result.push(self.dso_lever_icing()), // 2.4.9
                    // обледенение, лесовоз, неограниченный район
                    (true, true, true) => result.push(self.dso_lever_timber()),// 3.3.5            
                    (true, false, true) => { // обледенение, лесовоз, ограниченный район
                        result.push(self.dso_lever_icing());  // 2.4.9
                        result.push(self.dso_lever_timber()); // 3.3.5
                    },
                    // без обледенения, лесовоз
                    (false, _, true) => result.push(self.dso_lever_timber()), // 3.3.5
                    // без обледенения, все суда
                    (false, _, _) => result.push(self.dso_lever()), // 2.2.1.2             
                }
                result.append(&mut self.dso_lever_max_angle()); 
                //       if self.have_cargo {
                result.push(self.metacentric_height());
                //    }
                if let Ok(h_trans_fix) = self.metacentric_height.h_trans_fix() {
                    if self.navigation_area == NavigationArea::R2Rsn
                        || self.navigation_area == NavigationArea::R2Rsn45
                        || h_trans_fix.sqrt() / self.breadth > 0.08
                        || self.breadth / self.moulded_depth > 2.5
                    {
                        result.push(self.accelleration());
                    }
                }
                if self.have_container {
                    result.push(self.circulation());
                }
                if self.have_grain {
                    result.append(&mut self.grain());
                }
                result.push(self.metacentric_height_subdivision());
        
                // TODO        HeelMaximumLC, HeelFirstMaximumLC
                // out_data.push(CriterionData::new_result(CriterionID::HeelMaximumLC , self.lever_diagram.max_angles(), 1.);
        
                log::info!("Criterion end");
                let result = CriterionStabilityCtx {
                    criterion: result,
                }; 
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
impl std::fmt::Debug for WindEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
