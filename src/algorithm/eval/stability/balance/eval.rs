use crate::algorithm::entities::ship_model::stability_result::{BulkResult, LiquidResult};
use crate::algorithm::eval::stability::{StabilityBalanceCtx, StaticMassStabCtx};
use crate::infrostructure::ApiClient;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::ship_model::{
            BalanceStabilityQuery, ship_model::ShipModel, stability_result::BalanceStabilityResult,
        },
        eval::parameters::ParameterID,
    },
    kernel::{
        Eval,
        types::{RwLock, eval_result::EvalResult},
    },
    prelude::{ContextParamsWrite, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use std::sync::Arc;

///
/// Расчет равновесного положения судна
pub struct StabilityBalanceEval {
    dbg: Dbg,
    api_client: Arc<ApiClient>,    
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl StabilityBalanceEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        api_client: Arc<ApiClient>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "StabilityBalanceEval");
        Self {
            dbg,
            api_client,
            model,
            ctx: Box::new(ctx),
        }
    }
}
//
impl Eval<(), EvalResult> for StabilityBalanceEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_id = initial.ship_id.clone();
                let project_id = initial.project_id.clone();
                let hold_compartment = initial
                    .hold_compartment
                    .as_ref()
                    .ok_or(error.err("hold_compartment error: no data!"))?;
                self.model.write().update_hold_compartments(hold_compartment).map_err(|err| error.pass(err))?;
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let static_mass: StaticMassStabCtx = ctx.read();
                let water_density = voyage.density;
                // Расчет баланса для остойчивости в модели
                let stability_query = BalanceStabilityQuery {
                    water_density,
                    mass_const: static_mass.mass_const,
                    moment_const: static_mass.moment_const,
                    bulk: static_mass.bulk.clone(),
                    liquid: static_mass.liquid.clone(),
                    damaged_compartment: Vec::new(), //TODO: damaged_compartment, только для аварийного расчета
                };
                let result: BalanceStabilityResult = self
                    .model
                    .read()
                    .compute_stability(stability_query)
                    .map_err(|err| error.pass_with("model.compute_balance", err))?;
                /*
                                let liquid_data: HashMap<usize, String> = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                                        .liquid
                                        .as_ref()
                                        .ok_or(error.err("Read liquid error: no data!"))?
                                        .iter()
                                        .map(|(_, v)| (v.assignment_id, v.space_id.clone()))
                                        .collect();
                                result.liquid.iter().for_each(|v| println!("'{}' mass:{:.3} shift:{};", liquid_data.get(&v.assignment_id).unwrap(), v.mass, v.mass_shift.print()));

                                let bulk_data: HashMap<usize, String> = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                                        .bulk
                                        .as_ref()
                                        .ok_or(error.err("Read bulk error: no data!"))?
                                        .iter()
                                        .map(|(_, v)| (v.assignment_id, v.space_id.clone()))
                                        .collect();
                                result.bulk.iter().for_each(|v| println!("'{}' mass:{:.3} shift:{};", bulk_data.get(&v.assignment_id).unwrap(), v.mass, v.mass_shift.print()));

                                <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                                        .unit
                                        .as_ref()
                                        .ok_or(error.err("Read unit error: no data!"))?
                                        .iter()
                                        .map(|v| (v.space_id.clone(), v.mass, v.mass_shift()))
                                        .for_each(|v| println!("'{}' mass:{:.3} shift:{};", v.0, v.1, v.2.unwrap()));
                */
                ctx.write_params(ParameterID::DraughtMid, result.draught_mid);
                ctx.write_params(ParameterID::DraughtBow, result.draught_bow);
                ctx.write_params(ParameterID::DraughtStern, result.draught_stern);
                ctx.write_params(ParameterID::DraughtMean, result.draught_mean);
                ctx.write_params(ParameterID::TrimDeg, result.trim_degree);
                ctx.write_params(ParameterID::TrimMeter, result.trim_meter);
                ctx.write_params(ParameterID::Roll, result.heel);
                ctx.write_params(
                    ParameterID::TonesPerCm,
                    0.01 * result.area_wl * water_density,
                );
                ctx.write_params(ParameterID::MetacentricTransRad, result.rad_trans);
                ctx.write_params(ParameterID::MetacentricLongRad, result.rad_long);
                ctx.write_params(
                    ParameterID::CenterVolumeXFromStern,
                    result.displacement_center.x(),
                );
                ctx.write_params(
                    ParameterID::CenterWaterlineAreaXFromStern,
                    result.area_wl_center.x(),
                );
                ctx.write_params(ParameterID::CenterMassXFromStern, result.mass_center.x());
                ctx.write_params(ParameterID::CenterVolumeY, result.displacement_center.y());
                ctx.write_params(ParameterID::CenterVolumeZ, result.displacement_center.z());
                ctx.write_params(ParameterID::Displacement, result.mass);
                ctx.write_params(ParameterID::CenterMassX, result.mass_center.x());
                ctx.write_params(ParameterID::CenterMassY, result.mass_center.y());
                ctx.write_params(ParameterID::CenterMassZ, result.mass_center.z());
                let bulk = result.bulk.clone();
                send_bulk_param(
                    &self.dbg,
                    &ship_id,
                    &project_id,
                    &self.api_client,
                    &bulk,
                ).map_err(|err| error.pass(err))?;                
                let liquid = result.liquid.clone();
                send_liquid_param(
                    &self.dbg,
                    &ship_id,
                    &project_id,
                    &self.api_client,
                    &liquid,
                ).map_err(|err| error.pass(err))?;
                log::info!(
                    "StabilityBalance heel:{:.3} trim_degree:{:.3} trim_meter:{:.3} draught_mid:{:.3} displacement:{:.3} 
                    length_wl:{:.3} breadth_wl:{:.3} bow_area:{:.3} rad_trans:{:.3} rad_long:{:.3}
                    mass_center:{} displacement_center:{}\n",
                    result.heel,
                    result.trim_degree,
                    result.trim_meter,
                    result.draught_mid,
                    result.displacement,
                    result.length_wl,
                    result.breadth_wl,
                    result.bow_area,
                    result.rad_trans,
                    result.rad_long,
                    result.mass_center.print(),
                    result.displacement_center.print(),
                );
                let result = StabilityBalanceCtx {
                    heel: result.heel,
                    trim: result.trim_degree,
                    draught_mid: result.draught_mid,
                    mass_center: result.mass_center,
                    displacement: result.displacement,
                    bulk,
                    liquid,
                    length_wl: result.length_wl,
                    breadth_wl: result.breadth_wl,
                    bow_area: result.bow_area,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for StabilityBalanceEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StabilityBalanceEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
/// Запись данных расчета отсеков с сыпучими грузами
pub fn send_bulk_param(
    dbg: &Dbg,
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
    data: &Vec<BulkResult>,
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_bulk_param");
    log::info!("send_bulk_param begin");
    if data.is_empty() {
        return Err(error.err("empty data!"));
    }   
    let mut full_sql = "DO $$ BEGIN\n".to_owned();
    for data in data.iter() {
        full_sql += &format!(
            "UPDATE \"cargo_assignment\" SET centre_of_gravity_x = {}, centre_of_gravity_y = {}, centre_of_gravity_z = {} WHERE id = {};\n",
            data.mass_shift.x(), data.mass_shift.y(), data.mass_shift.z(), data.assignment_id
        );
        full_sql += &format!(
            "UPDATE \"cargo_assignment/compartment/bulk\" SET cargo_height = {}, allocated_shifting_moment = {} \
            WHERE id IN (SELECT bulk_cargo_assignment_id FROM \"cargo_assignment/compartment\" 
            WHERE id IN (SELECT compartment_cargo_assignment_id FROM \"cargo_assignment\" WHERE id = {}));\n",
            data.level, data.grain_moment, data.assignment_id
        );
        full_sql += &format!(
            "UPDATE \"space/compartment\" SET level = {}, volume = {} \
            WHERE id IN (SELECT compartment_id FROM bulk_cargo_view WHERE assignment_id = {});\n",
            data.level, data.volume, data.assignment_id
        );
    }
    full_sql += " END$$;";
 //   println!("{}", &full_sql);    
    api_client.fetch(&full_sql).map_err(|err| error.pass(err))?;
    log::info!("send_bulk_param end");
    Ok(())
}
/// Запись данных расчета отсеков с жидкими грузами
pub fn send_liquid_param(
    dbg: &Dbg,
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
    data: &Vec<LiquidResult>,
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_liquid_param");
    log::info!("send_liquid_param begin");
    if data.is_empty() {
        return Err(error.err("empty data!"));
    }   
    let mut full_sql = "DO $$ BEGIN\n".to_owned();
    for data in data.iter() {
        full_sql += &format!(
            "UPDATE \"cargo_assignment\" SET centre_of_gravity_x = {}, centre_of_gravity_y = {}, centre_of_gravity_z = {} WHERE id = {};\n",
            data.mass_shift.x(), data.mass_shift.y(), data.mass_shift.z(), data.assignment_id
        );
        full_sql += &format!(
            "UPDATE \"cargo_assignment/compartment/liquid\" SET cargo_height = {}, long_moment_of_inertia = {}, trans_moment_of_inertia = {} \
            WHERE id IN (SELECT liquid_cargo_assignment_id FROM \"cargo_assignment/compartment\" 
            WHERE id IN (SELECT compartment_cargo_assignment_id FROM \"cargo_assignment\" WHERE id = {}));\n",
            data.level, data.inertia_long_y, data.inertia_trans_x, data.assignment_id
        );
        full_sql += &format!(
            "UPDATE \"space/compartment\" SET level = {}, volume = {} \
            WHERE id IN (SELECT compartment_id FROM liquid_cargo_view WHERE assignment_id = {});\n",
            data.level, data.volume, data.assignment_id
        );
    }
    full_sql += " END$$;";
 //   println!("{}", &full_sql);    
    api_client.fetch(&full_sql).map_err(|err| error.pass(err))?;
    log::info!("send_liquid_param end");
    Ok(())
}
