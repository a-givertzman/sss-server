use super::mass_ctx::MassCtx;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
        entities::data::loads::{AssignmentType, UnitCargoType},
        eval::{BalanceCtx, IcingCtx, WettingCtx, parameters::ParameterID},
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{ContextParamsWrite, ContextRead, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
pub struct MassEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl MassEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "MassEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for MassEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_parameters = initial
                    .ship_parameters
                    .as_ref()
                    .ok_or(error.err("ship_parameters error: no data!"))?;
                let bounds = initial
                    .bounds
                    .as_ref()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let unit = initial
                    .unit
                    .as_ref()
                    .ok_or(error.err("Read unit error: no data!"))?
                    .iter()
                    .filter(|v| v.mass > 0.)
                    .collect::<Vec<_>>();
                let (unit, bulkhead): (Vec<_>, Vec<_>) = unit
                    .into_iter()
                    .partition(|v| v.cargo_type != UnitCargoType::GrainBulkhead);
                let lightship = initial
                    .load_constant
                    .as_ref()
                    .ok_or(error.err("Read lightship error: no data!"))?
                    .data();
                {
                    // суммарные массы по типам
                    let gaseous = initial
                        .gaseous
                        .as_ref()
                        .ok_or(error.err("Read gaseous error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                    let bulk = initial
                        .bulk
                        .as_ref()
                        .ok_or(error.err("Read bulk error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                    let liquid = initial
                        .liquid
                        .as_ref()
                        .ok_or(error.err("Read liquid error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                    let mass = |assigment_type: AssignmentType| {
                        gaseous
                            .iter()
                            .filter(|v| v.assigment_type == assigment_type)
                            .fold(0., |sum, v| sum + v.mass)
                            + bulk
                                .iter()
                                .filter(|v| v.assigment_type == assigment_type)
                                .fold(0., |sum, v| sum + v.mass)
                            + liquid
                                .iter()
                                .filter(|v| v.assigment_type == assigment_type)
                                .fold(0., |sum, v| sum + v.mass)
                            + unit
                                .iter()
                                .filter(|v| v.assigment_type == assigment_type)
                                .fold(0., |sum, v| sum + v.mass)
                    };
                    let ballast = mass(AssignmentType::Ballast);
                    let stores = mass(AssignmentType::Stores);
                    let cargo = mass(AssignmentType::CargoLoad);
                    let bulkhead = bulkhead.iter().fold(0., |sum, v| sum + v.mass);
                    let deadweight = ballast + stores + cargo + bulkhead; // Суммарная масса переменного груза
                    let lightship = lightship.iter().fold(0., |sum, v| sum + v.mass);
                    let icing = ContextRead::<IcingCtx>::read(&ctx).mass;
                    let wetting = ContextRead::<WettingCtx>::read(&ctx).mass;
                    let mass_sum = deadweight + lightship + wetting + icing;
                    ctx.write_params(ParameterID::Displacement, mass_sum);
                    ctx.write_params(ParameterID::MassBallast, ballast);
                    ctx.write_params(ParameterID::MassStores, stores);
                    ctx.write_params(ParameterID::MassBulkhead, bulkhead);
                    ctx.write_params(ParameterID::MassCargo, cargo);
                    ctx.write_params(ParameterID::MassDeadweight, deadweight);
                    ctx.write_params(ParameterID::MassLightship, lightship);
                    ctx.write_params(ParameterID::MassIcing, icing);
                    ctx.write_params(ParameterID::MassWetting, wetting);
                    log::info!(
                        "\t Mass ballast:{ballast}, stores:{stores}, bulkhead:{bulkhead}
                        cargo:{cargo}, deadweight:{deadweight}, lightship:{lightship},
                        icing:{icing}, wetting:{wetting} sum:{mass_sum}"
                    );
                }
                {
                    // распределения масс по типам
                    let balance: BalanceCtx = ctx.read();
                    let gaseous = initial
                        .gaseous
                        .as_ref()
                        .ok_or(error.err("Read gaseous error: no data!"))?;
                    let bulk = initial
                        .bulk
                        .as_ref()
                        .ok_or(error.err("Read bulk error: no data!"))?;
                    let liquid = initial
                        .liquid
                        .as_ref()
                        .ok_or(error.err("Read liquid error: no data!"))?;
                    let mut vec_hull = Vec::new();
                    let mut vec_equipment = Vec::new();
                    let mut vec_bulkhead = Vec::new();
                    let mut vec_ballast = Vec::new();
                    let mut vec_store = Vec::new();
                    let mut vec_cargo = Vec::new();
                    let mut vec_icing = ContextRead::<IcingCtx>::read(&ctx).mass_values;
                    let mut vec_wetting = ContextRead::<WettingCtx>::read(&ctx).mass_values;
                    let mut vec_sum = Vec::new();
                    let mut res: Vec<f64> = Vec::new();
                    for b in bounds.iter() {
                        vec_hull.push(
                            lightship
                                .iter()
                                .fold(0., |s, v| s + v.mass(b).unwrap_or(0.)),
                        );
                        vec_equipment.push(0.); //    TODO - сейчас в базе нет данных по equipment 
                        vec_bulkhead
                            .push(bulkhead.iter().fold(0., |s, v| s + v.mass(b).unwrap_or(0.)));
                        vec_ballast
                            .push(unit.iter().filter(|v| v.assigment_type == AssignmentType::Ballast).fold(0., |s, v| s + v.mass(b).unwrap_or(0.)));
                        vec_store
                            .push(unit.iter().filter(|v| v.assigment_type == AssignmentType::Stores).fold(0., |s, v| s + v.mass(b).unwrap_or(0.)));
                        vec_cargo
                            .push(unit.iter().filter(|v| v.assigment_type == AssignmentType::CargoLoad).fold(0., |s, v| s + v.mass(b).unwrap_or(0.)));
                    }

                    for v in balance.gaseous {

                    }

                    
                                    let mut vec_ballast = Vec::new();
                    let mut vec_store = Vec::new();
                    let mut vec_cargo = Vec::new();

                                        res.push(
                            hull + equipment + bulkhead + ballast + store + cargo + icing + wetting,
                        );                    
                }


                let result = MassCtx::new(data, values);

                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for MassEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval").field("dbg", &self.dbg).finish()
    }
}
