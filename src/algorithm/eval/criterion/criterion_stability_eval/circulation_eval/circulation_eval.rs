use super::circulation_ctx::CirculationCtx;
use crate::algorithm::context::context_access::ContextParamsRead;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{BalanceCtx, CriterionData, CriterionID, LeverDiagramCtx};
use crate::{
    prelude::*,
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия крена на циркуляции
pub struct CirculationEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl CirculationEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "CirculationEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for CirculationEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let voyage = initial
                    .voyage
                    .as_ref()
                    .ok_or(error.err("voyage error: no data!"))?;
                // Эксплуатационная скорость судна, m/s
                let v_0 = voyage.operational_speed;
                let balance: BalanceCtx = ctx.read();
                let d = ctx.read_params(ParameterID::DraughtMean);
                let l_wl = balance.length_wl;
                let moment_shift_z = ctx.read_params(ParameterID::CenterMassZ);
                let balance: BalanceCtx = ctx.read();
                let flooding_angle = balance.flooding_angle;
                // суммарная масса судна
                let mass = ctx.read_params(ParameterID::Displacement);
                // Плечо кренящего момента на циркуляции при скорости v, m/s
                let heel_lever = |v: f64| -> f64 {
                    // Кренящий момент на циркуляции
                    let m_r = 0.2 * (v * v * mass / l_wl) * (moment_shift_z - d / 2.).abs();
                    // Плечо кренящего момента на циркуляции
                    let l_r = m_r / mass;
                    log::trace!("Circulation angle v:{v} m_r:{m_r} l_r:{l_r}");
                    l_r
                };
                // Максимальная скорость при заданном угле крена
                let calculate_velocity = |target_angle: f64| -> Result<f64, Error> {
                    let mut current_vel = 10.; // m/s
                    let mut delta_vel = current_vel / 2.;
                    for _i in 0..20 {
                        let delta_angle = target_angle
                            - lever_diagram
                                .angle(heel_lever(current_vel))
                                .map_err(|err| error.pass_with("delta_angle", err))?
                                .first()
                                .copied()
                                .unwrap_or(90.);
                        if delta_angle.abs() < 0.001 {
                            break;
                        }
                        log::trace!(
                            "Circulation velocity src_angle:{target_angle} current_vel:{current_vel} delta_vel:{delta_vel} delta_angle:{delta_angle}"
                        );
                        current_vel = delta_vel * delta_angle.signum();
                        delta_vel /= 2.;
                    }
                    Ok(current_vel)
                };
                // Угла крена на циркуляции при скорости v_0, m/s
                let angle = match lever_diagram.angle(heel_lever(v_0)) {
                    Ok(angles) => angles.first().copied(),
                    Err(err) => {
                        let error = error.pass_with("angles", err);
                        log::error!("{error}");
                        let result = CirculationCtx {
                            data: CriterionData::new_error(
                                CriterionID::HeelTurning,
                                "Ошибка вычисления крена на циркуляции: ".to_owned()
                                    + &error.to_string(),
                            ),
                        };
                        return ctx.write(result);
                    }
                };
                let target = 16.0f64.min(flooding_angle / 2.);
                let result = if let Some(angle) = angle {
                    CriterionData::new_result(CriterionID::HeelTurning, angle, target)
                } else {
                    match calculate_velocity(target) {
                        Ok(velocity) => CriterionData::new_error(
                            CriterionID::HeelTurning,
                            format!(
                                "Крен {target} градусов, рекомендуемая скорость {} m/s');",
                                velocity,
                            ),
                        ),
                        Err(err) => {
                            let error = error.pass_with("calculate_velocity", err);
                            log::error!("{error}");
                            CriterionData::new_error(
                                CriterionID::HeelTurning,
                                "Ошибка вычисления рекомендуемой скорости: ".to_owned()
                                    + &error.to_string(),
                            )
                        }
                    }
                };
                let result = CirculationCtx { data: result };
                ctx.write(result)
                // TODO: В случаях, когда палубный груз контейнеров размещается только на крышках грузовых
                // люков, вместо угла входа кромки верхней палубы может приниматься меньший из углов
                // входа в воду верхней кромки комингса люка или входа контейнера в воду (в случае, когда
                // контейнеры выходят за пределы этого комингса).
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for CirculationEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CirculationEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
