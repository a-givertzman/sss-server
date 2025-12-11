//! Учет намокания груза
use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::{Bound, Moment, Position};
use crate::{
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use crate::WettingCtx;

///
/// Учет намокания палубного груза.  
/// При расчете намокания необходимо учитывать изменения водоизмещения и  
/// возвышения центра тяжести. Масса намокания и его моменты учитывается
/// при расчете прочности.
pub struct WettingEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl WettingEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WettingEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for WettingEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let bounds = match initial.bounds.as_ref() {
                    Some(data) => data,
                    None => return Err(error.err("Read bounds error: no data!")),
                };
                let unit = match initial.unit.as_ref() {
                    Some(data) => data,
                    None => return Err(error.err("Read unit error: no data!")),
                };
                let (mass, mass_moment) =
                    unit.iter()
                        .fold((0., Moment::zero()), |(res_mass, res_moment), v| {
                            let mass_shift = match v.mass_shift() {
                                Ok(v) => v,
                                Err(err) => {
                                    log::error!(
                                        "{}",
                                        error.pass_with(format!("{:?} mass_shift", v), err)
                                    );
                                    return (0., Position::zero());
                                }
                            };
                            let permeability = match v.permeability {
                                Some(v) => v,
                                None => 0.,
                            };
                            (
                                res_mass + v.mass * permeability,
                                res_moment + Moment::from_pos(mass_shift, v.mass * permeability),
                            )
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
                let mass_shift = if mass > 0. {
                    mass_moment.scale(1. / mass)
                } else {
                    Position::zero()
                };
                let result = WettingCtx {
                    mass,
                    mass_shift,
                    mass_values: mass_array,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for WettingEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WettingEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
