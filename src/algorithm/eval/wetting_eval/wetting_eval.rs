//! Учет намокания груза
use crate::algorithm::context::context_access::*;
use crate::algorithm::entities::{bound, Bound, Moment, Position};
use crate::{
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_core::{dbg::Dbg, error::Error};
use super::wetting_ctx::WettingCtx;

///
/// Учет намокания палубного груза.  
/// При расчете намокания необходимо учитывать изменения водоизмещения и  
/// возвышения центра тяжести. Масса намокания и его моменты учитывается
/// при расчете прочности.
pub struct WettingEval {
    dbg: Dbg,
    value: Option<WettingCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl WettingEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "WettingEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for WettingEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let bounds = match initial.bounds.as_ref() {
                    Some(data) => data,
                    None => {
                        return CtxResult::Err(error.err("Read bounds error: no data!"))
                    }
                };
                let unit = match initial.unit.as_ref() {
                    Some(data) => &data.data(),
                    None => {
                        return CtxResult::Err(error.err("Read unit error: no data!"))
                    }
                };
                let (mass, mass_moment) =
                    unit.iter()
                        .fold((0., Moment::zero()), |(res_mass, res_moment), v| {
                            match (v.mass_shift, v.permeability) {
                                (Some(v_mass_shift), Some(v_permeability)) => (
                                    res_mass + v.mass * v_permeability,
                                    res_moment
                                        + Moment::from_pos(
                                            v_mass_shift,
                                            v.mass * v_permeability,
                                        ),
                                ),
                                _ => (0., Position::zero()),
                            }
                        });
                let mass_array = bounds
                    .iter()
                    .map(|b| {
                        unit.iter()
                            .filter(|u| u.bound_x1.is_some() && u.bound_x2.is_some())
                            .map(|u| {
                                (
                                    Bound::new(u.bound_x1.unwrap(), u.bound_x2.unwrap()).ok(),
                                    u.mass,
                                    u.permeability.unwrap_or(0.),
                                )
                            })
                            .filter(|(bound, mass, permeability)| {
                                bound.is_some() && *mass > 0. && *permeability > 0.
                            })
                            .map(|(bound, mass, permeability)| {
                                bound.unwrap().part_ratio(b).unwrap_or(0.) * mass * permeability
                            })
                            .sum()
                    })
                    .collect();
                let mass_shift = mass_moment.scale(1. / mass);
                let result = WettingCtx {
                    mass,
                    mass_shift,
                    mass_values: mass_array,
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
impl std::fmt::Debug for WettingEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WettingEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
