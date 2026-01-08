use crate::algorithm::eval::stability::IcingStabCtx;
use crate::algorithm::eval::stability::static_mass::ctx::StaticMassCtx;
use crate::algorithm::context::context_access::ContextReadRef;
use crate::algorithm::entities::data::loads::UnitCargoType;
use crate::algorithm::entities::{Moment, Position};
use crate::algorithm::eval::{WettingCtx};
use crate::prelude::ContextRead;
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Расчет положения массы корпуса и грузов судна
pub struct StaticMassEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl StaticMassEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "StaticMassEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for StaticMassEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {                
                let icing: IcingStabCtx = ctx.read();
                let wetting: WettingCtx = ctx.read();
                let initial: &InitialCtx = ctx.read_ref();
                let bounds = initial
                    .bounds
                    .clone()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let ship_parameters = initial.ship_parameters.as_ref().ok_or(error.err("initial error: no ship_parameters!"))?;
                let shift_const = {
                    let const_mass_shift_x = ship_parameters
                        .get("LCG from middle")
                        .ok_or(error.err("Read const_mass_shift_x error: no data!"))?;
                    let const_mass_shift_y = ship_parameters
                        .get("TCG from CL")
                        .ok_or(error.err("Read const_mass_shift_y error: no data!"))?;
                    let const_mass_shift_z = ship_parameters
                        .get("VCG from BL")
                        .ok_or(error.err("Read const_mass_shift_z error: no data!"))?;
                    Position::new(*const_mass_shift_x, *const_mass_shift_y, *const_mass_shift_z)
                };
                let mass_const = ship_parameters
                        .get("LightShip Weight")
                        .ok_or(error.err("Read mass_const error: no data!"))?;    
                let bulk: Vec<_> = initial
                    .bulk
                    .clone()
                    .ok_or(error.err("Read bulk error: no data!"))?
                    .into_values()
                    .flat_map(|v| v.data())
                    .collect();
                let unit = initial
                    .unit
                    .clone()
                    .ok_or(error.err("Read unit error: no data!"))?
                    .into_iter()
                    .filter(|v| v.mass > 0.)
                    .collect::<Vec<_>>();
                let (mass_unit, shift_unit, grain_bulkhead) = {
                    let grain_bulkhead: Vec<_> = unit
                        .iter()
                        .filter(|v| {
                            v.cargo_type == UnitCargoType::GrainBulkhead && v.bound_x().is_ok()
                        })
                        .map(|v| v.bound_x().unwrap().center())
                        .flatten()
                        .collect();
                    let (mass_unit, moment_unit) = unit
                        .iter()
                        .filter_map(|v| match v.mass_shift() {
                            Ok(mass_shift) => {
                    //            println!("{} {} {:.?} ", v.cargo_name, v.mass, mass_shift);
                                Some((v.mass, mass_shift))
                            },
                            Err(err) => {
                                log::error!(
                                    "{}",
                                    error.pass_with(
                                        format!("{} {} mass_shift", v.space_id, v.cargo_name),
                                        err
                                    )
                                );
                                None
                            }
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

                    (mass_unit, moment_unit.to_pos(mass_unit), grain_bulkhead)
                };
            //    dbg!(mass_unit, shift_unit);
                let liquid: Vec<_> = initial
                    .liquid
                    .clone()
                    .ok_or(error.err("Read liquid error: no data!"))?
                    .into_values()
                    .flat_map(|v| v.data())
                    .collect();
                let (gaseous, mass_gaseous, shift_gaseous) = match initial.gaseous.clone() {
                    Some(data) => {
                        let (mass, shift) = data
                            .values()
                            .filter_map(|v| match v.mass_shift() {
                                Ok(mass_shift) => Some((v.mass, mass_shift)),
                                Err(err) => {
                                    log::error!(
                                        "{}",
                                        error.pass_with(format!("{:?} mass_shift", v), err)
                                    );
                                    None
                                }
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
                        let gaseous: Vec<_> = data.into_values().map(|v| v.data()).collect();
                        (gaseous, mass, shift)
                    }
                    None => return Err(error.err("Read gaseous error: no data!")),
                };
            //    dbg!(mass_gaseous, shift_gaseous);
                log::info!(
                    "StaticMass mass_const:{:.3} shift_const:{}
                    mass_unit:{:.3} shift_unit:{}
                    mass_gaseous:{:.3} shift_gaseous:{}
                    icing.mass:{:.3} shift_icing:{}
                    wetting.mass:{:.3} shift_wetting:{}",
                    mass_const, shift_const.print(),
                    mass_unit, shift_unit.print(),
                    mass_gaseous, shift_gaseous.print(),
                    icing.p_ice, icing.m_ice.print(),
                    wetting.mass, wetting.mass_shift.print()
                );           
                // Сумарный момент за вычетом смещяемых и насыпных груов
                let moment_const = Moment::from_pos(shift_const, *mass_const)
                    + Moment::from_pos(shift_unit, mass_unit)
                    + Moment::from_pos(shift_gaseous, mass_gaseous)
                    + Moment::from_pos(icing.m_ice, icing.p_ice)
                    + Moment::from_pos(wetting.mass_shift, wetting.mass);
                // Суммарная масса корпуса, обледенения с намоканием и грузов за вычетом смещяемых и насыпных грузов
                let mass_const = mass_const
                    + mass_unit
                    + mass_gaseous
                    + icing.p_ice
                    + wetting.mass;                   
                let result = StaticMassCtx {
                    mass_const,
                    moment_const,
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
impl std::fmt::Debug for StaticMassEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticMassEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
