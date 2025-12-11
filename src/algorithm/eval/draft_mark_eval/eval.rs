use crate::DraftMarkCtx;
use super::DraftMarkResult;
use crate::algorithm::context::context_access::{ContextParamsRead, ContextParamsWrite};
use crate::algorithm::entities::{Curve, ICurve};
use crate::algorithm::eval::parameters::ParameterID;
use crate::{
    prelude::ContextWrite,
    algorithm::context::context_access::ContextReadRef,
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет уровня заглубления для координат отметок заглубления на корпусе судна
pub struct DraftMarkEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl DraftMarkEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "DraftMarkEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for DraftMarkEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let data = initial.draft_mark.clone().unwrap();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .unwrap();
                let ship_length = *ship_parameters
                    .get("LBP")
                    .ok_or(error.err("No LBP in ship_parameters"))?;
                let roll = ctx.read_params(ParameterID::Roll).to_degrees();  
                let draught_bow = ctx.read_params(ParameterID::DraughtBow);    
                let draught_stern = ctx.read_params(ParameterID::DraughtStern);    
                let draught_mid = ctx.read_params(ParameterID::DraughtMid);
                let delta_draught = (draught_bow - draught_stern) / ship_length;
                let mut result = Vec::new();
                let draught_value = |pos_x: f64| -> f64 {
                    draught_mid
                    + delta_draught 
                    * pos_x
                };       
                for p in data.iter() {
                    if p.data.len() <= 2 {
                        log::error!("DraftMark eval error: p.data.len() <= 2, {}", p.name);
                        continue;
                    }
                    let mut z_fix: Vec<(f64, f64, f64, f64)> = Vec::new();
                    for v in p.data.iter() {
                        z_fix.push((
                            v.x(),
                            v.y(),
                            v.z(),
                            v.z() - v.y() * roll.sin() - draught_value(v.x()),
                        ));
                    }
                    z_fix.sort_by(|a, b| {
                        a.3.abs()
                            .partial_cmp(&b.3.abs())
                            .expect("DraftMark calculate error: partial_cmp!")
                    });
                    // Если все марки ниже или выше уровня воды
                    if z_fix[0].3.signum() == z_fix[1].3.signum() {
                        if z_fix[0].3.abs() < 0.001 {
                            // если уровень прямо на марке
                            result.push(DraftMarkResult::new(
                                p.criterion_id,
                                p.name.clone(),
                                z_fix[0].0,
                                z_fix[0].1,
                                Some(z_fix[0].2),
                            ));
                            ctx.write_params(ParameterID::from(p.criterion_id)?, z_fix[0].2);
                        } else {
                            result.push(DraftMarkResult::new(
                                p.criterion_id,
                                p.name.clone(),
                                z_fix[0].0,
                                z_fix[0].1,
                                None,
                            ));
                        }
                        continue;
                    }
                    // Интерполированные значения координат марок заглубления
                    let fix_x =
                        Curve::new_linear(&z_fix.iter().map(|&v| (v.3, v.0)).collect::<Vec<_>>()[..])
                            .map_err(|err| error.pass_with("fix_x", err))?
                            .value(0.)
                            .map_err(|err| error.pass_with("fix_x", err))?;
                    let fix_y =
                        Curve::new_linear(&z_fix.iter().map(|&v| (v.3, v.1)).collect::<Vec<_>>()[..])
                            .map_err(|err| error.pass_with("fix_y", err))?
                            .value(0.)
                            .map_err(|err| error.pass_with("fix_y", err))?;
                    let fix_z =
                        Curve::new_linear(&z_fix.iter().map(|&v| (v.3, v.2)).collect::<Vec<_>>()[..])
                            .map_err(|err| error.pass_with("fix_z", err))?
                            .value(0.)
                            .map_err(|err| error.pass_with("fix_z", err))?;
                    result.push(DraftMarkResult::new(
                        p.criterion_id,
                        p.name.clone(),
                        fix_x,
                        fix_y,
                        Some(fix_z),
                    ));
                    ctx.write_params(ParameterID::from(p.criterion_id)?, fix_z);
                }               
                let result = DraftMarkCtx {
                    data: result,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DraftMarkEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DraftMarkEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
