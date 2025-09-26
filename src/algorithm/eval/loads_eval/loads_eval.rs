use super::loads_ctx::LoadsCtx;
use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::data::loads::UnitCargoType;
use crate::algorithm::entities::{Moment, Position};
use crate::{
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{InitialCtx, ContextWrite}
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Расчет положения массы корпуса и грузов судна
pub struct LoadsEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl LoadsEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "LoadsEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for LoadsEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let shift_const = if let Some(ship_parameters) = initial.ship_parameters.as_ref() {
                    let const_mass_shift_x = *ship_parameters
                        .get("LCG from middle")
                        .ok_or(error.err("Read const_mass_shift_x error: no data!"))?;
                    let const_mass_shift_y = *ship_parameters
                        .get("TCG from CL")
                        .ok_or(error.err("Read const_mass_shift_y error: no data!"))?;
                    let const_mass_shift_z = *ship_parameters
                        .get("VCG from BL")
                        .ok_or(error.err("Read const_mass_shift_z error: no data!"))?;
                    Position::new(const_mass_shift_x, const_mass_shift_y, const_mass_shift_z)
                } else {
                    return Err(
                        error.err("Read const_mass_shift_z error: no ship_parameters!"),
                    );
                };
                let mass_const = match initial.load_constant.clone() {
                    Some(data) => data.data().iter().map(|v| v.mass).sum(),
                    None => return Err(error.err("Read load_constant error: no data!")),
                };
                let (bulk, mass_bulk, shift_bulk) = match initial.bulk.clone() {
                    Some(data) => {
                        let bulk: Vec<_> = data.iter().map(|v| v.data()).collect();
                        let (mass, shift) = data
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
                            );
                        (bulk, mass, shift)
                    },
                    None => return Err(error.err("Read bulk error: no data!")),
                };
                let (liquid, mass_liquid, shift_liquid) = match initial.liquid.clone() {
                    Some(data) => {
                        let liquid: Vec<_> = data.iter().map(|v| v.data()).collect();
                        let (mass, shift) = data
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
                            );
                        (liquid, mass, shift)
                    },
                    None => return Err(error.err("Read liquid error: no data!")),
                };
                let (mass_unit, shift_unit, grain_bulkhead) = match initial.unit.clone() {
                    Some(data) => {
                        let unit = data;
                        let grain_bulkhead: Vec<_> = unit
                            .iter()
                            .filter(|v| {
                                v.cargo_type == UnitCargoType::GrainBulkhead && v.bound_x().is_ok()
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
                    None => return Err(error.err("Read unit error: no data!")),
                };
                let (gaseous, mass_gaseous, shift_gaseous) = match initial.gaseous.clone() {
                    Some(data) => {
                        let gaseous: Vec<_> = data.iter().map(|v| v.data()).collect();
                        let (mass, shift) = data
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
                            );
                        (gaseous, mass, shift)
                    },
                  /*      data
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
                        ),*/
                    None => return Err(error.err("Read gaseous error: no data!")),
                };
                let result = LoadsCtx {
                    mass_const,
                    mass_bulk,
                    mass_liquid,
                    mass_unit,
                    mass_gaseous,
                    shift_const,
                    shift_unit,
                    shift_gaseous,
                    shift_liquid,
                    shift_bulk,
                    bulk,
                    liquid,
                    gaseous,
                    grain_bulkhead,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for LoadsEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadsEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
