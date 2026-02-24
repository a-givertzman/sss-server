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
                    mut dso,
                    mut entry_angle,
                    mut flooding_angle,
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
                //
                let process_dso = |dso: Vec<(f64, f64)>,
                                   entry_angle: Option<Vec<(f64, f64)>>,
                                   flooding_angle: Option<Vec<(f64, f64)>>|
                 -> Result<LeverDiagramCtx, Error> {
                    let dso_curve = Curve::new_linear(&dso)
                        .map_err(|e| error.pass_with("calculate curve", e))?;
                    // нахождение максимума диаграммы
                    let theta_max = {
                        let mut tmp_dso: Vec<&(f64, f64)> =
                            dso.iter().filter(|(a, _)| *a >= 0.).collect();
                        tmp_dso.sort_by(|(_, v1), (_, v2)| {
                            v2.partial_cmp(v1)
                                .expect("LeverDiagram calculate error: sort dso!")
                        });
                        let (theta_max, _max_value) = tmp_dso
                            .first()
                            .expect("LeverDiagram calculate error, no dso values!");
                        *theta_max
                    };
                    // нахождение углов максимумов и угла пересечения с 0
                    let max_angles: Vec<(f64, f64)> = {
                        let mut res = Vec::new();
                        let mut last_value = dso_curve
                            .value(0.)
                            .map_err(|e| error.pass_with("calculate last_value", e))?;
                        let mut last_value2 = last_value + 1.;
                        let mut last_angle = 0.;
                        for &(angle_deg, value) in dso.iter().filter(|(a, _)| *a >= 0.) {
                            if value < last_value && last_value > last_value2 {
                                res.push((last_angle, last_value));
                            }
                            if last_value != value {
                                last_value2 = last_value;
                                last_value = value;
                                last_angle = angle_deg
                            }
                        }
                        if res.is_empty() {
                            res.push((
                                theta_max,
                                dso_curve
                                    .value(theta_max)
                                    .map_err(|e| error.pass_with("calculate max_angles", e))?,
                            ));
                        }
                        res
                    };
                    //
                    let angle_zero = *crate::algorithm::eval::stability::lever_diagram::ctx::angle(
                        theta_max, &dso_curve, 0.,
                    )
                    .map_err(|e| error.pass_with("calculate angle_zero", e))?
                    .first()
                    .ok_or(error.err("calculate angle_zero no angles"))?;
                    //
                    let ddo = {
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
                        ddo
                    };
                    // Поиск входа в воду отверстий и палубы как пересечения с 0
                    let find_zero_angle = |v: Vec<(f64, f64)>| -> Result<f64, Error> {
                        let v: Vec<_> = v
                            .into_iter()
                            .filter(|&(a, _v)| a >= 0.)
                            .map(|(a, v)| (v, a))
                            .collect();
                        Curve::new_linear(&v)
                            .map_err(|err| error.pass(err))?
                            .value(0.)
                            .map_err(|err| error.pass(err))
                    };
                    let entry_angle = if let Some(angle) = entry_angle {
                        find_zero_angle(angle).map_err(|err| error.pass_with("entry_angle", err))?
                    } else {
                        0. // для записи в базу не считаем
                    };
                    let flooding_angle = if let Some(angle) = flooding_angle {
                        find_zero_angle(angle)
                            .map_err(|err| error.pass_with("flooding_angle", err))?
                    } else {
                        0. // для записи в базу не считаем
                    };
                    let result = LeverDiagramCtx {
                        dso,
                        dso_curve,
                        ddo,
                        theta_max,
                        max_angles,
                        entry_angle,
                        flooding_angle,
                    };
                    Ok(result)
                };
                // Исходная диаграмма для записи результата в базу данных.
                let result = process_dso(dso.clone(), None, None)
                    .map_err(|err| error.pass_with("process_dso for db", err))?;
                let raw_diagram = result
                    .dso
                    .iter()
                    .filter(|(a, _)| a.fract().abs() < 0.001)
                    .zip(result.ddo.iter())
                    .map(|((a1, v1), (_, v2))| (*a1, *v1, *v2))
                    .collect::<Vec<_>>();
                // Диаграмма для дальнейшего расчета.
                // Проверяем нулевое плечо если оно положительно переворачиваем его
                // чтобы в расчете учитывался худший случай крена.
                // плечо для нулевого угла
                let lever_zero = dso
                    .iter()
                    .find(|(a, _)| *a == 0.)
                    .ok_or(error.err("calculate lever_zero error!"))?
                    .1;
                // знак статического угла крена
                // если крен на левый борт то переворачиваем диаграммы
                if lever_zero > 0. {
                    let reverse = |mut v: Vec<(f64, f64)>| -> Vec<(f64, f64)> {
                        v = v.into_iter().map(|(a, v)| (-a, -v)).collect();
                        v.sort_by(|(a1, _), (a2, _)| {
                            a1.partial_cmp(a2)
                                .expect("LeverDiagram calculate error: sort!")
                        });
                        v
                    };
                    dso = reverse(dso);
                    entry_angle = reverse(entry_angle);
                    flooding_angle = reverse(flooding_angle);
                };
                let result = process_dso(dso.clone(), Some(entry_angle), Some(flooding_angle))
                    .map_err(|err| error.pass_with("process_dso for db", err))?;
                let transformed_diagram = result
                    .dso
                    .iter()
                    .filter(|(a, _)| a.fract().abs() < 0.001)
                    .zip(result.ddo.iter())
                    .map(|((a1, v1), (_, v2))| (*a1, *v1, *v2))
                    .collect::<Vec<_>>();
                send_stability_diagram(
                    &self.dbg,
                    &ship_id,
                    &project_id,
                    &self.api_client,
                    raw_diagram,
                    transformed_diagram,
                )
                .map_err(|err| error.pass(err))?;

                ctx.write_params(ParameterID::Roll, heel);
                ctx.write_params(ParameterID::OpenDeckEdgeImmersionAngle, result.entry_angle);
                ctx.write_params(ParameterID::AngleOfDownFlooding, result.flooding_angle);
                /*            log::info!(
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
                );*/
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
    raw_diagram: Vec<(f64, f64, f64)>,
    transformed_diagram: Vec<(f64, f64, f64)>,
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_stability_diagram");
    log::info!("send_stability_diagram begin");
    if raw_diagram.is_empty() {
        return Err(error.err("empty raw_diagram!"));
    }
    if raw_diagram.len() != transformed_diagram.len() {
        return Err(error.err("raw_diagram.len() != transformed_diagram.len(!"));
    }
    let data: Vec<_> = raw_diagram
        .iter()
        .zip(transformed_diagram.iter())
        .map(|(v1, v2)| (v1.0, v1.1, v1.2, v2.1, v2.2))
        .collect();
    let values_list: Vec<String> = data
        .iter()
        .map(|(angle, raw_dso, raw_ddo, transformed_dso, transformed_ddo)| {
            format!("({ship_id}, {project_id}, {angle}, {raw_dso}, {raw_ddo}, {transformed_dso}, {transformed_ddo})")
        })
        .collect();
    let full_sql = format!(
        "DO $$ BEGIN \
        DELETE FROM stability_diagram \
        WHERE ship_id = {ship_id} AND project_id IS NOT DISTINCT FROM {project_id}; \
        INSERT INTO stability_diagram \
          (ship_id, project_id, angle, raw_value_dso, raw_value_ddo, transformed_value_dso, transformed_value_ddo) \
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
