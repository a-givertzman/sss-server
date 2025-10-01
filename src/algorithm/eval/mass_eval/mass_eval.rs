use super::mass_ctx::MassCtx;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef, entities::data::loads::{AssignmentType, UnitCargoType},
        eval::{IcingCtx, WettingCtx, parameters::ParameterID},
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
                let unit = initial
                        .unit
                        .as_ref()
                        .ok_or(error.err("Read unit error: no data!"))?
                        .iter()
                        .filter(|v| v.mass > 0.)
                        .collect::<Vec<_>>();  
                let mass_type = |assigment_type: AssignmentType| {
                    gaseous
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type)
                        .fold(0., |sum,v| sum + v.mass) + 
                    bulk
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type)
                        .fold(0., |sum,v| sum + v.mass) +
                    liquid
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type)
                        .fold(0., |sum,v| sum + v.mass) +
                    unit
                        .iter()
                        .filter(|v| v.assigment_type == assigment_type && v.cargo_type != UnitCargoType::GrainBulkhead)
                        .fold(0., |sum, v| sum + v.mass)                                             
                };
                let ballast = mass_type(AssignmentType::Ballast);
                let stores = mass_type(AssignmentType::Stores);
                let cargo = mass_type(AssignmentType::CargoLoad);
                let bulkhead = unit
                        .iter()
                        .filter(|v| v.cargo_type == UnitCargoType::GrainBulkhead)
                        .fold(0., |sum,v| sum + v.mass);              
                let deadweight = ballast + stores + cargo + bulkhead; // Суммарная масса переменного груза
                let lightship = initial.load_constant.as_ref()
                        .ok_or(error.err("Read liquid error: no data!"))?
                        .data()
                        .iter()
                        .fold(0., |sum, v| sum + v.mass);             
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
