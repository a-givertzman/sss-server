use crate::algorithm::entities::Position;
use crate::algorithm::entities::model_cached::DsoResult;
use crate::algorithm::entities::ship_model::BalanceStabilityQuery;
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::stability::{LeverDiagramCtx, StabilityBalanceCtx, StaticMassStabCtx};
use crate::infrostructure::ApiClient;
use crate::kernel::Eval;
use crate::kernel::types::Arc;
use crate::prelude::{ContextParamsWrite, ContextReadRef, InitialCtx};
use crate::{
    algorithm::{context::context_access::ContextRead, entities::math::curve::*, eval::zg::Zg},
    kernel::types::eval_result::EvalResult,
    prelude::ContextWrite,
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::sync::RwLock;

///
/// Диаграмма плеч статической и динамической остойчивости
pub struct LeverDiagramEval {
    dbg: Dbg,
    api_client: Arc<ApiClient>,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl LeverDiagramEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        api_client: Arc<ApiClient>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "LeverDiagramEval");
        Self {
            dbg,
            api_client,
            model,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for LeverDiagramEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix.clone()) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_id = initial.ship_id.clone();
                let project_id = initial.project_id.clone();
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                let static_mass: StaticMassStabCtx = ctx.read();
                let stability_result: StabilityBalanceCtx = ctx.read();
                // Расчет баланса для остойчивости в модели
                let query = BalanceStabilityQuery {
                    water_density: voyage.density,
                    mass_const: static_mass.mass_const,
                    moment_const: static_mass.moment_const,
                    bulk: static_mass.bulk.clone(),
                    liquid: static_mass.liquid.clone(),
                    damaged_compartment: Vec::new(), //TODO: damaged_compartment, только для аварийного расчета
                };
                let cg = if let Zg(Some(z_g_fix)) = z_g_fix {
                    Position::new(
                        stability_result.mass_center.x(),
                        stability_result.mass_center.y(),
                        z_g_fix,
                    )
                } else {
                    stability_result.mass_center
                };
                let DsoResult {
                    heel,
                    dso,
                    entry_angle,
                    flooding_angle,
                } = self
                    .model
                    .read()
                    .compute_dso(
                        stability_result.heel,
                        stability_result.trim,
                        stability_result.draught_mid,
                        cg,
                        query,
                    )
                    .map_err(|err| error.pass_with("model.compute_dso", err))?;
                let dso_curve =
                    Curve::new_linear(&dso).map_err(|e| error.pass_with("calculate curve", e))?;
                // нахождение максимума диаграммы
                let mut tmp_dso: Vec<&(f64, f64)> = dso.iter().filter(|(a, _)| *a >= 0.).collect();
                tmp_dso.sort_by(|(_, v1), (_, v2)| {
                    v2.partial_cmp(v1)
                        .expect("LeverDiagram calculate error: sort dso!")
                });
                let (theta_max, _max_value) = tmp_dso
                    .first()
                    .expect("LeverDiagram calculate error, no dso values!");
                let theta_max = *theta_max;
                //     log::trace!( "{}", format!("LeverDiagram calculate max_angle:{theta_max} max_value:{max_value}"));
                // нахождение углов максимумов и угла пересечения с 0
                let mut max_angles: Vec<(f64, f64)> = Vec::new();
                let mut last_value = dso_curve
                    .value(0.)
                    .map_err(|e| error.pass_with("calculate last_value", e))?;
                let mut last_value2 = last_value + 1.;
                let mut last_angle = 0.;
                for &(angle_deg, value) in dso.iter().filter(|(a, _)| *a >= 0.) {
                    if value < last_value && last_value > last_value2 {
                        max_angles.push((last_angle, last_value));
                    }
                    if last_value != value {
                        last_value2 = last_value;
                        last_value = value;
                        last_angle = angle_deg
                    }
                }
                if max_angles.is_empty() {
                    max_angles.push((
                        theta_max,
                        dso_curve
                            .value(theta_max)
                            .map_err(|e| error.pass_with("calculate max_angles", e))?,
                    ));
                }
                //
                let angle_zero = crate::algorithm::eval::stability::lever_diagram::ctx::angle(
                    theta_max, &dso_curve, 0.,
                );
                let angle_zero = *angle_zero
                    .map_err(|e| error.pass_with("calculate angle_zero", e))?
                    .first()
                    .ok_or(error.err("calculate angle_zero no angles"))?;
                let mut ddo = Vec::new();
                for &(angle_deg, _) in dso.iter().filter(|(a, _)| a.fract().abs() < 0.001) {
                    let value = if angle_deg < angle_zero {
                        dso_curve
                            .integral(angle_deg, angle_zero)
                            .map_err(|e| error.pass_with("calculate ddo", e))?
                            .to_radians()
                    } else if angle_deg > angle_zero {
                        dso_curve
                            .integral(angle_zero, angle_deg)
                            .map_err(|e| error.pass_with("calculate ddo", e))?
                            .to_radians()
                    } else {
                        0.
                    };
                    ddo.push((angle_deg, value));
                }
                let diagram = dso
                    .iter()
                    .filter(|(a, _)| a.fract().abs() < 0.001)
                    .zip(ddo.iter())
                    .map(|((a1, v1), (_, v2))| (*a1, *v1, *v2))
                    .collect::<Vec<_>>();
                ctx.write_params(ParameterID::Roll, heel);
                ctx.write_params(ParameterID::OpenDeckEdgeImmersionAngle, entry_angle);
                ctx.write_params(ParameterID::AngleOfDownFlooding, flooding_angle);
                let result = LeverDiagramCtx {
                    dso,
                    dso_curve,
                    ddo,
                    theta_max,
                    max_angles,
                    entry_angle,
                    flooding_angle,
                };
                log::info!(
                    "LeverDiagram theta_max:{}\n max_angles [angle l]:{}\n entry_angle:{}\n flooding_angle:{}\n diagram [angle dso ddo]:{}\n",
                    result.theta_max,
                    result
                        .max_angles
                        .iter()
                        .fold(String::new(), |s, v| s + &format!(
                            " ({:.3} {:.3})",
                            v.0, v.1,
                        )),
                    result.entry_angle,
                    result.flooding_angle,
                    diagram.iter().fold(String::new(), |s, v| s + &format!(
                        "\n{:.3} {:.3} {:.3}",
                        v.0, v.1, v.2
                    )),
                );
                send_stability_diagram(&self.dbg, &ship_id, &project_id, &self.api_client, diagram)
                    .map_err(|err| error.pass(err))?;
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for LeverDiagramEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeverDiagramEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
/// Запись данных расчета плечей остойчивости в БД
pub fn send_stability_diagram(
    dbg: &Dbg,
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
    data: Vec<(f64, f64, f64)>,
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_stability_diagram");
    log::info!("send_stability_diagram begin");
    if data.is_empty() {
        return Err(error.err("empty data!"));
    }   
    let values_list: Vec<String> = data
        .iter()
        .map(|(angle, value_dso, value_ddo)| 
            format!("({ship_id}, {project_id}, {angle}, {value_dso}, {value_ddo})")
        ).collect();    
    let full_sql = format!(
        "DO $$ BEGIN \
        DELETE FROM stability_diagram \
        WHERE ship_id = {ship_id} AND project_id IS NOT DISTINCT FROM {project_id}; \
        INSERT INTO stability_diagram \
          (ship_id, project_id, angle, value_dso, value_ddo) \
        VALUES \
          {values_str}; \
        END $$;",
        values_str = values_list.join(", ")
    );
 //   println!("{}", &full_sql);
    api_client.fetch(&full_sql).map_err(|err| error.pass(err))?;
    log::info!("send_stability_diagram end");
    Ok(())
}
