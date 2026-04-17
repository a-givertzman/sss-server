use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
        entities::{
            Moment, Position,
            data::loads::{AssignmentType, UnitCargoType},
        },
        eval::{
            parameters::ParameterID,
            stability::StabilityBalanceCtx,
        },
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextParamsWrite, ContextRead, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
pub struct DynamicMassStabEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl DynamicMassStabEval {
    //
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "DynamicMassStabEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for DynamicMassStabEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
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
                let sability_balance: StabilityBalanceCtx = ctx.read();
                let bulk = sability_balance.bulk;
                let liquid = sability_balance.liquid;
                let unit = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                    .unit
                    .clone()
                    .ok_or(error.err("Read unit error: no data!"))?
                    .into_iter()
                    .filter(|v| v.mass > 0.)
                    .collect::<Vec<_>>();
                let (unit, bulkhead): (Vec<_>, Vec<_>) = unit
                    .into_iter()
                    .partition(|v| v.cargo_type != UnitCargoType::GrainBulkhead);
                let gaseous = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .gaseous
                        .as_ref()
                        .ok_or(error.err("Read gaseous error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                let calc = |assigment_type: AssignmentType| {
                    let (gaseous_mass, gaseous_moment) = gaseous
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type)
                        .fold((0., Moment::zero()), |(mass_sum, moment_sum), v| {
                            (
                                mass_sum + v.mass,
                                moment_sum
                                    + Moment::from_pos(
                                        v.mass_shift().unwrap_or(Position::zero()),
                                        v.mass,
                                    ),
                            )
                        });
                    let (bulk_mass, bulk_moment) = bulk
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type)
                        .fold((0., Moment::zero()), |(mass_sum, moment_sum), v| {
                            (
                                mass_sum + v.mass,
                                moment_sum
                                    + Moment::from_pos(
                                        v.mass_shift,
                                        v.mass,
                                    ),
                            )
                        });
                    let (liquid_mass,liquid_moment) = liquid
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type)
                        .fold((0., Moment::zero()), |(mass_sum, moment_sum), v| {
                            (
                                mass_sum + v.mass,
                                moment_sum
                                    + Moment::from_pos(
                                        v.mass_shift,
                                        v.mass,
                                    ),
                            )
                        });
                    let (unit_mass, unit_moment) = unit
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type)
                        .fold((0., Moment::zero()), |(mass_sum, moment_sum), v| {
                            (
                                mass_sum + v.mass,
                                moment_sum
                                    + Moment::from_pos(
                                        v.mass_shift().unwrap_or(Position::zero()),
                                        v.mass,
                                    ),
                            )
                        });
                    let mass = gaseous_mass + bulk_mass + liquid_mass + unit_mass;
                    let moment = gaseous_moment + bulk_moment + liquid_moment + unit_moment;
                    let shift = moment.to_pos(mass);
                    (mass, moment, shift)
                };
                let (ballast_mass, ballast_moment, ballast_shift) = calc(AssignmentType::Ballast);
                let (stores_mass, stores_moment, stores_shift) = calc(AssignmentType::Stores);
                let (cargo_mass, cargo_moment, cargo_shift) = calc(AssignmentType::CargoLoad);
                let (bulkhead_mass, bulkhead_moment) = bulkhead
                    .iter()
                    .fold((0., Moment::zero()), |(mass_sum, moment_sum), v| {
                        (
                            mass_sum + v.mass,
                            moment_sum
                                + Moment::from_pos(
                                    v.mass_shift().unwrap_or(Position::zero()),
                                    v.mass,
                                ),
                        )
                    });
                let bulkhead_shift = bulkhead_moment.to_pos(bulkhead_mass);
                let deadweight_mass = ballast_mass + stores_mass + cargo_mass + bulkhead_mass; // Суммарная масса переменного груза
                let deadweight_shift = (ballast_moment + stores_moment + cargo_moment + bulkhead_moment).to_pos(deadweight_mass);  
                log::info!(
                    "\t Mass ballast:({ballast_mass}, {}), stores:({stores_mass}, {}), bulkhead:({bulkhead_mass}, {})
                    cargo:({cargo_mass}, {}), deadweight:({deadweight_mass}, {})", 
                    ballast_shift.print(), stores_shift.print(), 
                    bulkhead_shift.print(), cargo_shift.print(), 
                    deadweight_shift.print()
                ); 
                ctx.write_params(ParameterID::MassLightship, *mass_const);  
                ctx.write_params(ParameterID::MassLightshipX, shift_const.x());
                ctx.write_params(ParameterID::MassLightshipY, shift_const.y());
                ctx.write_params(ParameterID::MassLightshipZ, shift_const.z());                   
                ctx.write_params(ParameterID::MassBulkhead, bulkhead_mass); 
                ctx.write_params(ParameterID::MassBulkheadX, bulkhead_shift.x());
                ctx.write_params(ParameterID::MassBulkheadY, bulkhead_shift.y());
                ctx.write_params(ParameterID::MassBulkheadZ, bulkhead_shift.z());
                ctx.write_params(ParameterID::MassBallast, ballast_mass);
                ctx.write_params(ParameterID::MassBallastX, ballast_shift.x());
                ctx.write_params(ParameterID::MassBallastY, ballast_shift.y());
                ctx.write_params(ParameterID::MassBallastZ, ballast_shift.z());
                ctx.write_params(ParameterID::MassStores, stores_mass);
                ctx.write_params(ParameterID::MassStoresX, stores_shift.x());
                ctx.write_params(ParameterID::MassStoresY, stores_shift.y());
                ctx.write_params(ParameterID::MassStoresZ, stores_shift.z());
                ctx.write_params(ParameterID::MassCargo, cargo_mass);
                ctx.write_params(ParameterID::MassCargoX, cargo_shift.x());
                ctx.write_params(ParameterID::MassCargoY, cargo_shift.y());
                ctx.write_params(ParameterID::MassCargoZ, cargo_shift.z());
                ctx.write_params(ParameterID::MassDeadweight, deadweight_mass);
                ctx.write_params(ParameterID::CenterMassDeadweightX, deadweight_shift.x());
                ctx.write_params(ParameterID::CenterMassDeadweightY, deadweight_shift.y());
                ctx.write_params(ParameterID::CenterMassDeadweightZ, deadweight_shift.z());
                Ok(ctx)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DynamicMassStabEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval").field("dbg", &self.dbg).finish()
    }
}
