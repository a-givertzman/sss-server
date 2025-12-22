use crate::algorithm::context::context_access::{ContextRead, ContextReadRef};
use crate::algorithm::entities::Moment;
use crate::algorithm::eval::{IcingCtx, IcingStabCtx, StaticAreaCtx};
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Учет обледенения судна
pub struct IcingEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl IcingEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
}
//
impl Eval<(), EvalResult> for IcingEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let bounds = match initial.bounds.clone() {
                    Some(data) => data,
                    None => return Err(error.err("Read bounds error: no data!")),
                };
                let area_strength: StaticAreaCtx = ctx.read();
                let icing_stab: IcingStabCtx = ctx.read();
                let mut mass_values = Vec::new();
                //    let mut mass_moment_x_sum = 0.;
                for (i, _) in bounds.iter().enumerate() {
                    /*    let current_x = match bound.center() {
                        Some(data) => data,
                        None => {
                            return Err(error.err(format!("bound.center error: no center for bound {i}")));
                        }
                    };*/
                    let current_area_h = match area_strength.area_h_values.get(i) {
                        Some(&data) => data,
                        None => {
                            return Err(error.err(format!(
                                "area_strength.area_h.get error: no value for bound {i}"
                            )));
                        }
                    };
                    let current_area_v = match area_strength.area_v_values.get(i) {
                        Some(&data) => data,
                        None => {
                            return Err(error.err(format!(
                                "area_strength.area_v.get error: no value for bound {i}"
                            )));
                        }
                    };
                    let current_area_timber_h =
                        match area_strength.area_timber_icing_h_values.get(i) {
                            Some(&data) => data,
                            None => {
                                return Err(error.err(format!(
                                    "area_strength.area_timber_h.get error: no value for bound {i}"
                                )));
                            }
                        };
                    let current_mass = current_area_h * icing_stab.mass_desc_h
                        + current_area_timber_h
                            * (icing_stab.mass_timber_h - icing_stab.mass_desc_h)
                        + current_area_v * (1. + icing_stab.coef_v_ds_area) * icing_stab.mass_v;
                    //    mass_moment_x_sum += current_mass * current_x;
                    mass_values.push(current_mass);
                }
                let mass_v =
                    area_strength.area_v * (1. + icing_stab.coef_v_ds_area) * icing_stab.mass_v;
                let mass_h = area_strength.area_h * icing_stab.mass_desc_h;
                let mass_timber_h = area_strength.area_timber_icing_h
                    * (icing_stab.mass_timber_h - icing_stab.mass_desc_h);
                let mass_sum = mass_v + mass_h + mass_timber_h;
                assert!((mass_sum - mass_values.iter().sum::<f64>()).abs() < 0.0001);
                let moment_v = Moment::from_pos(area_strength.area_v_shift, mass_v);
                let moment_h = Moment::from_pos(area_strength.area_h_shift, mass_h);
                let moment_timber =
                    Moment::from_pos(area_strength.area_timber_icing_h_shift, mass_timber_h);
                let moment_sum = moment_v + moment_h + moment_timber;
                let mass_shift = moment_sum.to_pos(mass_sum);
                let result = IcingCtx {
                    mass: mass_sum,
                    mass_shift,
                    mass_values,
                };
                log::info!(
                    "Icing mass:{:.3} shift:({:.3}, {:.3},{:.3})",
                    result.mass,
                    result.mass_shift.x(),
                    result.mass_shift.y(),
                    result.mass_shift.z()
                );
                log::debug!(
                    "Icing values:{}\n",
                    result
                        .mass_values
                        .iter()
                        .fold(String::new(), |s, v| s + &format!("{:.3} ", v))
                );
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for IcingEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingEval").field("dbg", &self.dbg).finish()
    }
}
