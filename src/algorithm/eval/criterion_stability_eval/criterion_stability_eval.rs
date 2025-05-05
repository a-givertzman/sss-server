use super::criterion_stability_ctx::CriterionStabilityCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef}, entities::data::{ship_type::ShipType, NavigationArea}, eval::{AccelerationCtx, DSOAngleMaxCtx, DSOAreaCtx, DSOIcingMaxCtx, DSOMaxCtx, DSOTimberMaxCtx, IcingStabCtx, MetacentricHeightCtx, MinMetacentricHeightCtx, StaticAngleCtx, WheatherCtx}
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite, CtxResult,
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
                match (have_icing, navigation_area == NavigationArea::Unrestricted, ship_type == ShipType::TimberCarrier) {
                    // обледенение, не лесовоз, неограниченный район
                    (true, true, false) => {
                        data.push(ContextRead::<DSOMaxCtx>::read(&ctx).data) // 2.2.1.2    
                    },         
                    // обледенение, не лесовоз, ограниченный район
                    (true, false, false) => {                        
                        data.push(ContextRead::<DSOIcingMaxCtx>::read(&ctx).data) // 2.4.9
                    }, 
                    // обледенение, лесовоз, неограниченный район
                    (true, true, true) => {
                        data.push(ContextRead::<DSOTimberMaxCtx>::read(&ctx).data)// 3.3.5  
                    },           
                    (true, false, true) => { // обледенение, лесовоз, ограниченный район
                        data.push(ContextRead::<DSOIcingMaxCtx>::read(&ctx).data);  // 2.4.9
                        data.push(ContextRead::<DSOTimberMaxCtx>::read(&ctx).data); // 3.3.5   
                    },
                    // без обледенения, лесовоз
                    (false, _, true) => {
                        data.push(ContextRead::<DSOTimberMaxCtx>::read(&ctx).data) // 3.3.5
                    }
                    // без обледенения, все суда
                    (false, _, _) => {
                        data.push(ContextRead::<DSOMaxCtx>::read(&ctx).data) // 2.2.1.2   
                    },          
                }
                data.append(&mut ContextRead::<DSOAngleMaxCtx>::read(&ctx).data);   
                //       if self.have_cargo {
                data.push(ContextRead::<MinMetacentricHeightCtx>::read(&ctx).data);
                //    }
                
                    if navigation_area == NavigationArea::R2Rsn
                        || navigation_area == NavigationArea::R2Rsn45
                        || h_trans_fix.sqrt() / breadth > 0.08
                        || breadth / moulded_depth > 2.5
                    {
                        data.push(ContextRead::<AccelerationCtx>::read(&ctx).data);
                    }

                if have_container {
                    data.push(ContextRead::<CirculationCtx>::read(&ctx).data self.circulation());
                }
                if have_grain {
                    data.append(&mut self.grain());
                }
                data.push(self.metacentric_height_subdivision());
        
                // TODO        HeelMaximumLC, HeelFirstMaximumLC
                // data.push(CriterionData::new_result(CriterionID::HeelMaximumLC , self.lever_diagram.max_angles(), 1.);
        
                info!("Criterion end");
                data



                let result = CriterionStabilityCtx {
                    data,
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
impl std::fmt::Debug for CriterionStabilityEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CriterionStabilityEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
