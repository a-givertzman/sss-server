use crate::algorithm::eval::strength::result::{IResults, Results};
use crate::algorithm::eval::strength::{DynamicMassCtx, StrengthBalanceCtx};
use crate::infrostructure::ApiClient;
use crate::kernel::types::Arc;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, InitialCtx},
};
use sal_3dlib_core::math::*;
use sal_core::{dbg::Dbg, error::Error};

///
/// Результаты расчета по прочности
pub struct ResultStrEval {
    dbg: Dbg,
    api_client: Arc<ApiClient>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ResultStrEval {
    //
    pub fn new(
        parent: impl Into<String>,
        api_client: Arc<ApiClient>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ResultStrEval");
        Self {
            dbg,
            api_client,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for ResultStrEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_id = initial.ship_id.clone();
                let project_id = initial.project_id.clone();
                let bounds = initial
                    .bounds
                    .as_ref()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let strength_limits = initial
                    .strength_limits
                    .as_ref()
                    .ok_or(error.err("initial error: no strength_limits!"))?;
                let mass: DynamicMassCtx = ctx.read();
                let mass_values = mass.values;
                let balance: StrengthBalanceCtx = ctx.read();
                let volume_values = balance.displacement_distr;
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let water_density = voyage.density;
                let gravity_g = 9.81;
                if mass_values.len() != volume_values.len() {
                    let error = error.err("mass_values.len() != volume_values.len()");
                    log::error!("{error}");
                    return Err(error);
                }
                let mut total_force = mass_values.clone();
                let mut displacement_mass = volume_values;
                displacement_mass.mul_single(water_density);
                //    println!("\n\n mass qnt:{} sum: {}\n", mass_values.len(), mass_values.iter().sum::<f64>());  mass_values.iter().for_each(|b| print!("{:.3} ", b));
                //    println!("\n\n volume qnt:{} sum: {}\n", volume_values.len(), volume_values.iter().sum::<f64>());  volume_values.iter().for_each(|b| print!("{:.3} ", b));
                total_force.sub_vec(&displacement_mass)?;
                total_force.mul_single(gravity_g);
                let shear_force = total_force.sum_above();
                let mut delta_x = vec![0.];
                delta_x.append(&mut bounds.iter().map(|b| b.length().unwrap_or(0.)).collect());
                let values: Vec<_> = shear_force.iter().zip(delta_x.iter()).collect();
                let mut bending_moment = vec![0.];
                for i in 1..(values.len()) {
                    let (v1, _) = values[i - 1];
                    let (v2, dx) = values[i];
                    bending_moment.push(bending_moment[i - 1] + (v1 + v2) * dx / 2.);
                }
                let results = Results::new();
                let names = [
                    "value_mass_hull",
                    "value_mass_equipment",
                    "value_mass_bulkhead",
                    "value_mass_ballast",
                    "value_mass_store",
                    "value_mass_cargo",
                    "value_mass_icing",
                    "value_mass_wetting",
                    "value_mass_sum",
                ];
                let add = |name: &str| -> Result<(), Error> {
                    let _: () = results.add_values(name, mass.data.get(name).ok_or(error.err(name))?);
                    Ok(())
                };
                let (_, errors): (Vec<_>, Vec<_>) =
                    names.into_iter().map(add).partition(Result::is_ok);
                let err_mess = errors
                    .into_iter()
                    .map(Result::unwrap_err)
                    .fold(String::new(), |acc, err| format!("{acc}\n\t error: {err}"));
                if !err_mess.is_empty() {
                    log::error!("{}", error.err(&err_mess));
                    return Err(error.err(err_mess));
                }
                results.add_values("value_displacement", &displacement_mass);
                results.add_values("value_total_force", &total_force);
                results.add_results("value_shear_force", &shear_force);
                results.add_results("value_bending_moment", &bending_moment);
                let (start_x, end_x): (Vec<_>, Vec<_>) = bounds
                    .iter()
                    .map(|b| (b.start().unwrap_or(0.), b.end().unwrap_or(0.)))
                    .unzip();
                results.add_values("start_x", &start_x);
                results.add_values("end_x", &end_x);
                let mut frame_x = start_x;
                frame_x.push(*end_x.last().unwrap_or(&0.));
                results.add_results("frame_x", &frame_x);
                let (sf_min, sf_max, bm_min, bm_max) = strength_limits.data();
                let compute_percent = |result: f64, limit: f64| -> f64 {
                    if limit != 0. {
                        result * 100. / limit
                    } else {
                        100.
                    }
                };
                let calculate = |result: &Vec<f64>,
                                limit_min: Vec<(f64, f64)>,
                                limit_max: Vec<(f64, f64)>|
                -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), Error> {
                    let limit_min = Curve::new_linear(&limit_min).map_err(|err| error.pass_with("limit_min", err))?;
                    let limit_min: Vec<_> = frame_x
                        .iter()
                        .map(|&x| limit_min.value(x).unwrap_or(0.))
                        .collect();
                    let limit_max = Curve::new_linear(&limit_max).map_err(|err| error.pass_with("limit_max", err))?;
                    let limit_max: Vec<_> = frame_x
                        .iter()
                        .map(|&x| limit_max.value(x).unwrap_or(0.))
                        .collect();
                    let (percent, status) = result
                        .iter()
                        .zip(limit_min.iter())
                        .zip(limit_max.iter())
                        .map(|((&result, &limit_min), &limit_max)| {
                            let percent = if result < 0. {
                                compute_percent(result, limit_min)
                            } else {
                                compute_percent(result, limit_max)
                            };
                            let status = if percent < 100. { 1. } else { 0. }; // 1 - true, 0 - false
                            (percent, status)
                        })
                        .unzip();
                    Ok((limit_min, limit_max, percent, status))
                };
                let (sf_min, sf_max, sf_percent, sf_status) =
                    calculate(&shear_force, sf_min, sf_max)?;
                results.add_results("limit_low_shear_force", &sf_min);
                results.add_results("limit_high_shear_force", &sf_max);
                results.add_results("percent_shear_force", &sf_percent);
                results.add_results("status_shear_force", &sf_status);
                let (bm_min, bm_max, bm_percent, bm_status) =
                    calculate(&bending_moment, bm_min, bm_max)?;
                results.add_results("limit_low_bending_moment", &bm_min);
                results.add_results("limit_high_bending_moment", &bm_max);
                results.add_results("percent_bending_moment", &bm_percent);
                results.add_results("status_bending_moment", &bm_status);
                send_values(
                    &self.dbg,
                    &ship_id,
                    &project_id,
                    &self.api_client,
                    results.take_values(),
                )
                .map_err(|err| error.pass(err))?;
                send_results(
                    &self.dbg,
                    &ship_id,
                    &project_id,
                    &self.api_client,
                    results.take_results(),
                )
                .map_err(|err| error.pass(err))?;
                Ok(ctx)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ResultStrEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultStrEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
/// Запись промежуточных данных расчета прочности в БД
fn send_values(
    dbg: &Dbg,
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
    data: (Vec<String>, Vec<Vec<f64>>),
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_values");
    log::info!("send_values begin");
    let (names, values) = data;
    let values_list: Vec<String> = values
        .iter()
        .map(|row| {
            let row_data = row.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("({ship_id}, {project_id}, {row_data})")
        })
        .collect();
    let full_sql = format!(
        "DO $$ BEGIN \
        DELETE FROM result_strength_values \
        WHERE ship_id = {ship_id} AND project_id IS NOT DISTINCT FROM {project_id}; \
        INSERT INTO result_strength_values (ship_id, project_id, {names_list}) \
        VALUES {values_str}; \
        END $$;",
        names_list = names.join(", "),
        values_str = values_list.join(", ")
    );
 //   println!("{}", &full_sql);
    api_client.fetch(&full_sql).map_err(|err| error.pass(err))?;
    log::info!("send_values end");
    Ok(())
}
/// Запись результата расчета прочности в БД
fn send_results(
    dbg: &Dbg,
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
    data: (Vec<String>, Vec<Vec<f64>>),
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_results");
    log::info!("send_results begin");
    let (names, values) = data;
    let values_str: Vec<_> = values
        .iter()
        .map(|row| {
            let row_values: Vec<_> = row
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    if names[i].contains("status") {
                        format!("{}", *n == 1.) // 1 -
                    } else {
                        format!("{n}")
                    }
                })
                .collect();
            format!(" ({ship_id}, {project_id}, {})", row_values.join(", "))
        })
        .collect();
    let full_sql = &format!(
        "DO $$ BEGIN \
        DELETE FROM result_strength_force_and_moment \
        WHERE ship_id = {ship_id} AND project_id IS NOT DISTINCT FROM {project_id}; \
        INSERT INTO result_strength_force_and_moment \
         (ship_id, project_id, {names_list}) \
        VALUES \
         {values_list}; \
        END$$;",
        names_list = names.join(", "),
        values_list = values_str.join(", ")
    ).to_owned();
  //  println!("{}", &full_sql);
    api_client.fetch(full_sql).map_err(|err| error.pass(err))?;
    log::info!("send_results end");
    Ok(())
}
