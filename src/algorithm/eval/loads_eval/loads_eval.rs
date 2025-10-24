use super::loads_ctx::LoadsCtx;
use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::data::loads::UnitCargoType;
use crate::algorithm::entities::ship_model::{BulkData, GaseousData, LiquidData};
use crate::algorithm::entities::{Bound, Bounds, Moment, Position};
use crate::{
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{ContextWrite, InitialCtx},
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
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
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
                let bounds = initial
                    .bounds
                    .clone()
                    .ok_or(error.err("initial error: no bounds!"))?;
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
                    return Err(error.err("Read const_mass_shift_z error: no ship_parameters!"));
                };
                let load_constant = initial
                    .load_constant
                    .clone()
                    .ok_or(error.err("Read load_constant error: no data!"))?
                    .data();
                let (lightship_values, lightship_bounds): (Vec<_>, Vec<_>) = load_constant
                    .into_iter()
                    .map(|v| (v.mass, Bound::Value(v.bound_x1, v.bound_x2)))
                    .unzip();
                let mass_const = lightship_values.iter().sum();
                let lightship_bounds = Bounds::new(lightship_bounds)
                    .map_err(|err| error.pass_with("Bounds::new", err))?;
                let vec_hull = bounds
                    .intersect(&lightship_bounds, &lightship_values)
                    .map_err(|err| error.pass_with("bounds.intersect", err))?;
                let vec_equipment = vec![0.; bounds.len_qnt()]; //    TODO - сейчас в базе нет данных по equipment 
                let bulk: Vec<_> = initial
                    .bulk
                    .clone()
                    .ok_or(error.err("Read bulk error: no data!"))?
                    .into_values()
                    .flat_map(|v| v.data())
                    .collect();
                let liquid: Vec<_> = initial
                    .liquid
                    .clone()
                    .ok_or(error.err("Read liquid error: no data!"))?
                    .into_values()
                    .flat_map(|v| v.data())
                    .collect();
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
                        let (mass, shift) = data
                            .values()
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
                        let gaseous: Vec<_> = data
                            .into_values()
                            .flat_map(|v| {
                                Some(GaseousData {
                                    assigned_id: v.assigned_id, // ID assigned
                                    space_id: v.space_id,       // ID помещения
                                    mass: v.mass,
                                })
                            })
                            .collect();
                        (gaseous, mass, shift)
                    }
                    None => return Err(error.err("Read gaseous error: no data!")),
                };


                vec_hull.add_vec()
                let add = |v1: &Vec<f64>, v2: &Vec<f64>| -> Vec<f64> {
                        v1.iter().zip(v2.iter()).map(|(v1, v2)| v1 + v2).collect()
                    };



                let result = LoadsCtx {
                    mass_const,
                    mass_unit,
                    mass_gaseous,
                    shift_const,
                    shift_unit,
                    shift_gaseous,
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
        f.debug_struct("LoadsEval").field("dbg", &self.dbg).finish()
    }
}
