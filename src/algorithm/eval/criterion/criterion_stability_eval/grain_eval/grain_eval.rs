use super::grain_ctx::GrainCtx;
use crate::algorithm::context::context_access::{ContextParamsRead, ContextParamsWrite};
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{CriterionData, CriterionID, LeverDiagramCtx};
use crate::{
    BalanceCtx, ContextWrite, algorithm::context::context_access::ContextRead,
    kernel::{eval::Eval, types::eval_result::EvalResult},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия при перевозки навалочных смещаемых грузов
pub struct GrainEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl GrainEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "GrainEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for GrainEval {
    fn eval(&mut self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(mut ctx) => {
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let balance: BalanceCtx = ctx.read();
                let m_grain = balance
                    .bulk
                    .iter()
                    .map(|v| v.moment )
                    .sum();
                let mass = ctx.read_params(ParameterID::Displacement);
                let balance: BalanceCtx = ctx.read();
                let flooding_angle = balance.flooding_angle;
                let mut results = Vec::new();
                let lambda_0 = m_grain / mass;
                // Первая точка апроксимирующей прямой
                let first_point_ab = (0.0f64, lambda_0);
                // Вторая точка апроксимирующей прямой
                let second_point_ab = (40.0f64, 0.8 * lambda_0);
                // Изменение апроксимирующей прямой на один градус угла крена
                let delta_ab =
                    (second_point_ab.1 - first_point_ab.1) / (second_point_ab.0 - first_point_ab.0);
                let precision = 0.01; // Точность определения пересечения в градусах
                // Точка пересечения кривых. Проходим по кривой плечей и ищем точку пересечения как
                // точку, в которой значение кривой плеч момента зерна меньше чем значение dso
                // Если точка отсутствует (момент от зерна слишком большй) то принимаем
                // максимальный угол при расчете диаграммы
                let max_angles = lever_diagram.max_angles();
                let angle = match max_angles.first() {
                    Some((angle, _lever)) => angle,
                    None => {
                        let error = error.err("no max max_angle");
                        log::error!("{error}");
                        results.push(CriterionData::new_error(
                            CriterionID::HeelGrainDisplacement,
                            "Ошибка вычисления расчетного и максимально допустимого угла крена от смещения зерна крена: ".to_owned() + &error.to_string(),
                        ));
                        let result = GrainCtx { data: results };
                        return ctx.write(result);
                    }
                };
                let max_i: f64 = angle / precision;
                let max_i = max_i.ceil() as usize;
                let theta_grain_angle = (0..=max_i)
                    .find(|i| {
                        let i = *i as f64;
                        // значение угла крена в текущей точке
                        let angle = i * precision;
                        // значение апроксимирующей прямой плеч момента зерна в текущей точке
                        let lever_ab = lambda_0 + delta_ab * angle;
                        // значение восстанавливающего момента в текущей точке
                        let lever_dso = lever_diagram.lever_moment(angle).unwrap_or(f64::MIN);
                        //    log::trace!("\t Grain area first_angle i:{i} angle:{angle} lever_ab:{lever_ab} lever_dso:{lever_dso}");
                        lever_dso >= lever_ab
                    })
                    .unwrap_or(max_i) as f64
                    * precision;
                let target_grain_angle = flooding_angle.min(12.);
                let theta_grain_lever = lever_diagram.lever_moment(theta_grain_angle)?;
                // угол соответствующий максимальной разности между ординатами двух кривых
                let mut angles: Vec<(f64, f64)> = (0..=max_i)
                    .map(|i| {
                        let i = i as f64;
                        // значение угла крена в текущей точке
                        let angle = i * precision;
                        // значение апроксимирующей прямой плеч момента зерна в текущей точке
                        let lever_ab = lambda_0 + delta_ab * angle;
                        // значение восстанавливающего момента в текущей точке
                        let lever_dso = lever_diagram.lever_moment(angle).unwrap_or(lever_ab);
                        (angle, lever_dso - lever_ab)
                    })
                    .collect();
                angles
                    .sort_by(|v1, v2| v1.1.partial_cmp(&v2.1).expect("Grain calculate cmp error"));
                let angle_delta_max = angles.last().unwrap_or(&(0., 0.)).0;
                let second_angle = flooding_angle.min(40.).min(angle_delta_max);
                // Площадь кривой восстанавливающих плеч
                let dso_area = lever_diagram
                    .dso_area(theta_grain_angle, second_angle)
                    .map_err(|err| error.pass_with("dso_area", err))?;
                // Площадь кривой кренящих плеч от смещения зерна
                let first_grain_lever = lever_diagram
                    .lever_moment(theta_grain_angle)
                    .map_err(|err| error.pass_with("first_grain_lever", err))?;
                let second_grain_lever = lambda_0 + delta_ab * second_angle;
                let grain_area = (first_grain_lever
                    + (second_grain_lever - first_grain_lever) / 2.)
                    * (second_angle - theta_grain_angle).to_radians();
                let result_area = dso_area - grain_area;
                let theta_grain40 = lever_diagram
                    .lever_moment(second_angle)
                    .map_err(|err| error.pass_with("theta_grain40", err))?;
                log::trace!("\t Grain area m_grain:{m_grain} lambda_0:{lambda_0} 
                    first_point_ab:{:?} second_point_ab:{:?}
                    first_angle:{theta_grain_angle} angle_delta_max:{angle_delta_max} second_angle:{second_angle} 
                    delta_ab:{delta_ab} dso_area:{dso_area} first_grain_lever:{first_grain_lever} second_grain_lever:{second_grain_lever}
                    grain_area:{grain_area} result_area:{result_area}", first_point_ab, second_point_ab);
                ctx.write_params(
                    ParameterID::HeelingMomentDueToTheTransverseShiftOfGrain,
                    m_grain,
                );
                // Первая точка прямой
                ctx.write_params(
                    ParameterID::HeelingLeverDueToTheTransverseShiftOfGrainWithZeroDifference,
                    lambda_0,
                );
                // Первая точка пересечения
                ctx.write_params(
                    ParameterID::HeelingAngleDueToTheTransverseShiftOfGrain,
                    theta_grain_angle,
                );
                ctx.write_params(
                    ParameterID::HeelingLeverDueToTheTransverseShiftOfGrain,
                    theta_grain_lever,
                );
                // Вторая точка прямой
                ctx.write_params(ParameterID::HeelingAngleWithMaximumDifference, second_angle);
                ctx.write_params(
                    ParameterID::HeelingLeverOfDSOWithMaximumDifference,
                    second_grain_lever,
                );
                // Точка пересечения угла и ДСО
                ctx.write_params(
                    ParameterID::HeelingLeverOfCurveWithMaximumDifference,
                    theta_grain40,
                );
                // Остаточная площадь
                ctx.write_params(ParameterID::GrainArea, result_area);
                let mut results = Vec::new();
                results.push(CriterionData::new_result(
                    CriterionID::HeelGrainDisplacement,
                    theta_grain_angle,
                    target_grain_angle,
                ));
                results.push(CriterionData::new_result(
                    CriterionID::AreaLcGrainDisplacement,
                    grain_area,
                    0.075,
                ));
                let result = GrainCtx { data: results };
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
impl std::fmt::Debug for GrainEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrainEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
