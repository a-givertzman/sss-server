use super::lever_diagram_ctx::LeverDiagramCtx;
use crate::{
    ContextWrite, CtxResult,
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{Bound, Moment, Position, data::loads::UnitCargoType},
        eval::IcingTimberCtx,
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ship_model::model_link::{IModelLink, ModelLink},
};
use sal_core::{dbg::Dbg, error::Error};
pub(crate) const MAX_ANGLE_CALC: f64 = 60.;
///
/// Диаграмма плеч статической и динамической остойчивости
pub struct LeverDiagramEval {
    dbg: Dbg,
    model: ModelLink,
    value: Option<LeverDiagramCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl LeverDiagramEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: ModelLink,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "LeverDiagramEval");
        Self {
            dbg,
            model,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for LeverDiagramEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let pantocaren = match self.model.pantocaren() {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(error.pass_with( "Read pantocaren error", err));
                    }
                };
                let parameters: Parameters = ctx.read(); 
                let z_g_fix = parameters.get(ParameterID::CenterMassZFix).ok_or(CtxResult::Err(error.err("calculate z_g_fix error: no CenterMassZFix in parameters")))?;
                let y_g = parameters.get(ParameterID::CenterMassY).ok_or(CtxResult::Err(error.err("calculate y_g error: no CenterMassY in parameters")))?;
                let y_c = parameters.get(ParameterID::CenterVolumeY).ok_or(CtxResult::Err(error.err("calculate y_c error: no CenterVolumeY in parameters")))?;
                let delta_y = y_g - y_c;
                log::info!(
                    "LeverDiagram calculate z_g_fix:{z_g_fix} y_g:{y_g} y_c:{y_c} delta_y:{delta_y}"
                );
                let max = (MAX_ANGLE_CALC * 10.) as i32;
                let min = -max;
                let roll = (min..=max).map(|i| i as f64 * 0.1).collect::<Vec<f64>>();
                let levers = pantocaren.clone();
                let theta =
                    &|(angle_deg, lever)| -> Result<(f64, f64), Error> {
                        let angle_rad = angle_deg.to_radians();
                        let v1 = lever;
                        let v2 = z_g_fix * angle_rad.sin();
                        let v3 = delta_y * angle_rad.cos();
                        let value = v1 - v2 - v3;
                        //    if angle_deg.fract() == 0. {
                        //        log::info!("{}", format!("LeverDiagram calculate расчет диаграммы: theta deg:{angle_deg}, l_k:{v1}, z_g_fix*sin(theta):{v2}, (y_g - y_c)*cos(theta):{v3}, l:{value}"));
                        //    }
                        Ok((angle_deg, value))
                    };
                let mut dso = levers
                    .into_iter()
                    .filter_map(|v| theta(v).ok())
                    .collect::<Vec<(f64, f64)>>();
                // плечо для нулевого угла
                let lever_zero = dso
                    .iter()
                    .find(|(a, _)| *a == 0.)
                    .ok_or(CtxResult::Err(error.err("calculate lever_zero error!")))?.1;
                // знак статического угла крена
                let mut angle_zero_signum = 1.; // если крен на левый борт то переворачиваем диаграмму
                if lever_zero > 0. {
                    dso = dso.into_iter().map(|(a, v)| (-a, -v)).collect();
                    dso.sort_by(|(a1, _), (a2, _)| {
                        a1.partial_cmp(a2)
                            .expect("LeverDiagram calculate error: sort dso!")
                    });
                    angle_zero_signum = -1.; // сохраняем знак угла
                }
                // нахождение максимума диаграммы
                let mut tmp_dso: Vec<&(f64, f64)> = dso.iter().filter(|(a, _)| *a >= 0.).collect();
                tmp_dso.sort_by(|(_, v1), (_, v2)| {
                    v2.partial_cmp(v1)
                        .expect("LeverDiagram calculate error: sort dso!")
                });
                let curve = Curve::new_linear(&dso).map_err(|e| CtxResult::Err(error.pass_with("calculate curve", e)))?;
                let mut angle = tmp_dso
                    .first()
                    .expect("LeverDiagram calculate error, no dso values!")
                    .0;
                let mut max_angle = angle;
                let mut value = curve.value(angle).map_err(|e| CtxResult::Err(error.pass_with("calculate value", e)))?;
                let mut max_value = value;
                let mut delta_angle = 1.;
                for _i in 0..10 {
                    let delta_angle_l = angle - delta_angle;
                    let value_l = curve.value(delta_angle_l).map_err(|e| {
                        Error::FromString(format!("LeverDiagram calculate value_l error: {}", e))
                    })?;
                    let delta_angle_r = angle + delta_angle;
                    let value_r = curve.value(delta_angle_r).map_err(|e| {
                        Error::FromString(format!("LeverDiagram calculate value_r error: {}", e))
                    })?;
                    if value_l >= value_r {
                        value = value_l;
                        angle -= delta_angle;
                    } else {
                        value = value_r;
                        angle += delta_angle;
                    }
                    if value >= max_value {
                        max_value = value;
                        max_angle = angle;
                    } else {
                        angle = max_angle;
                    }
                    delta_angle *= 0.5;
                    //    log::info!("{}", format!("LeverDiagram calculate max_angle: value:{value} angle:{angle} max_value:{max_value} max_angle:{max_angle} delta_angle:{delta_angle} i:{_i} "));
                }
                log::trace!(
                    "{}",
                    format!("LeverDiagram calculate max_angle:{max_angle}")
                );
                self.theta_max = Some(max_angle);
                self.dso = Some(dso.clone());
                self.dso_curve = Some(curve.clone());
                // нахождение углов максимумов и угла пересечения с 0
                let mut max_angles: Vec<(f64, f64)> = Vec::new();
                let mut last_value = curve.value(0.).map_err(|e| {
                    Error::FromString(format!("LeverDiagram calculate last_value error: {}", e))
                })?;
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
                        max_angle,
                        curve.value(max_angle).map_err(|e| {
                            Error::FromString(format!(
                                "LeverDiagram calculate max_angles error: {}",
                                e
                            ))
                        })?,
                    ));
                }
                self.max_angles = Some(max_angles);
                //
                let angle_zero = *self
                    .angle(0.)
                    .map_err(|e| {
                        Error::FromString(format!("LeverDiagram calculate self.angle error: {}", e))
                    })?
                    .first()
                    .unwrap_or(&0.);
                let mut ddo = Vec::new();
                for &(angle_deg, _) in dso.iter().filter(|(a, _)| a.fract().abs() < 0.001) {
                    let value = if angle_deg < angle_zero {
                        curve
                            .integral(angle_deg, angle_zero)
                            .map_err(|e| {
                                Error::FromString(format!(
                                    "LeverDiagram calculate ddo error: {}",
                                    e
                                ))
                            })?
                            .to_radians()
                    } else if angle_deg > angle_zero {
                        curve
                            .integral(angle_zero, angle_deg)
                            .map_err(|e| {
                                Error::FromString(format!(
                                    "LeverDiagram calculate ddo error: {}",
                                    e
                                ))
                            })?
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
                /*   log::trace!(
                    "LeverDiagram calculate z_g_fix:{z_g_fix} angle_zero:{}",
                    angle_zero * angle_zero_signum,
                );*/
                log::trace!("LeverDiagram calculate diagram: [angle dso ddo]:");
                for &(angle, dso, ddo) in diagram.iter() {
                    log::trace!("{angle} {dso} {ddo};");
                }
                self.diagram = Some(diagram);
                self.ddo = Some(ddo);
                self.parameters
                    .add(ParameterID::Roll, angle_zero * angle_zero_signum);

                let result = LeverDiagramCtx {
                    area_v,
                    moment_v,
                    moment_h,
                    moment_timber_h,
                    delta_moment_timber_h,
                };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for LeverDiagramEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeverDiagramEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
