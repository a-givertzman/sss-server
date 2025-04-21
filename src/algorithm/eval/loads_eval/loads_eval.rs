use super::loads_ctx::LoadsCtx;
use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::data::loads::UnitCargoType;
use crate::algorithm::entities::{Moment, Position};
use crate::{
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Расчет положения массы корпуса и грузов судна
pub struct LoadsEval {
    dbg: Dbg,
    value: Option<LoadsCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl LoadsEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "LoadsEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for LoadsEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let shift_const = if let Some(ship_parameters) =
                    initial.ship_parameters.as_ref()
                {
                    let const_mass_shift_x = *match ship_parameters.get("LCG from middle") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(error.err("Read const_mass_shift_x error: no data!"))
                        }
                    };
                    let const_mass_shift_y = *match ship_parameters.get("TCG from CL") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(error.err("Read const_mass_shift_y error: no data!"))
                        }
                    };
                    let const_mass_shift_z = *match ship_parameters.get("VCG from BL") {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(error.err("Read const_mass_shift_z error: no data!"))
                        }
                    };
                    Position::new(const_mass_shift_x, const_mass_shift_y, const_mass_shift_z)
                } else {
                    return CtxResult::Err(error.err("Read const_mass_shift_z error: no ship_parameters!"));
                };
                let mass_const = match initial.load_constant.clone() {
                    Some(data) => data.data().iter().map(|v| v.mass).sum(),
                    None => {
                        return CtxResult::Err(error.err("Read load_constant error: no data!"))
                    }
                };
                let bulk: Vec<_> = match initial.bulk.clone() {
                    Some(data) => data.iter().map(|v| v.data()).collect(),
                    None => {
                        return CtxResult::Err(error.err("Read bulk error: no data!"))
                    }
                };
                let liquid: Vec<_> = match initial.liquid.clone() {
                    Some(data) => data.iter().map(|v| v.data()).collect(),
                    None => {
                        return CtxResult::Err(error.err("Read liquid error: no data!"))
                    }
                };

                let (mass_unit, shift_unit, grain_bulkhead) = match initial.unit.clone() {
                    Some(data) => {
                        let unit = data;
                        let grain_bulkhead: Vec<_> = unit
                            .iter()
                            .filter(|v| {
                                v.cargo_type == UnitCargoType::GrainBulkhead
                                    && v.bound_x().is_ok()
                            })
                            .map(|v| v.bound_x().unwrap().center())
                            .flatten()
                            .collect();
                        let (mass_unit, shift_unit) = unit
                            .iter()
                            .filter_map(|v| match v.mass_shift() {
                                Ok(mass_shift) => Some((v.mass, mass_shift)),
                                Err(_) => None,
                            })
                            .fold(
                                (0., Moment::zero()),
                                |(mass_sum, moment_sum), (mass, mass_shift)| {
                                    (
                                        mass_sum + mass,
                                        moment_sum + Moment::from_pos(mass_shift, mass),
                                    )
                                },
                            );
                        (mass_unit, shift_unit, grain_bulkhead)
                    }
                    None => {
                        return CtxResult::Err(error.err("Read unit error: no data!"))
                    }
                };
                let (mass_gaseous, shift_gaseous) = match initial.gaseous.clone() {
                    Some(data) => data
                        .iter()
                        .filter_map(|v| match v.mass_shift {
                            Some(mass_shift) => Some((v.mass, mass_shift)),
                            None => None,
                        })
                        .fold(
                            (0., Moment::zero()),
                            |(mass_sum, moment_sum), (mass, mass_shift)| {
                                (
                                    mass_sum + mass,
                                    moment_sum + Moment::from_pos(mass_shift, mass),
                                )
                            },
                        ),
                    None => {
                        return CtxResult::Err(error.err("Read gaseous error: no data!"))
                    }
                };
                let result = LoadsCtx {
                    mass_const,
                    mass_bulk: bulk.iter().fold(0., |sum, v| sum + v.mass),
                    mass_liquid: liquid.iter().fold(0., |sum, v| sum + v.mass),
                    mass_unit,
                    mass_gaseous,
                    shift_const,
                    shift_unit,
                    shift_gaseous,
                    bulk,
                    liquid,
                    grain_bulkhead,
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
impl std::fmt::Debug for LoadsEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadsEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
