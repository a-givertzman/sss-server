use super::lever_diagram_ctx::LeverDiagramCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextParamsWrite, ContextRead},
        entities::math::curve::*,
        eval::{parameters::ParameterID, BalanceCtx},
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Диаграмма плеч статической и динамической остойчивости
pub struct LeverDiagramEval {
    dbg: Dbg,
  //  model: Link,
    value: Option<LeverDiagramCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl LeverDiagramEval {
    ///
    pub fn new(
        parent: impl Into<String>,
  //      model: Link,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "LeverDiagramEval");
        Self {
            dbg,
    //        model,
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
            CtxResult::Ok(mut ctx) => {
        //        let ctx = self.ctx.take().unwrap();
                let balance: BalanceCtx = ctx.read();
                let pantocaren = &balance.pantocaren; 
                let z_g_fix = ctx.read_params(ParameterID::CenterMassZFix);
                let y_g = ctx.read_params(ParameterID::CenterMassY);
                let y_c = ctx.read_params(ParameterID::CenterVolumeY);
                let delta_y = y_g - y_c;
                log::info!(
                    "LeverDiagram calculate z_g_fix:{z_g_fix} y_g:{y_g} y_c:{y_c} delta_y:{delta_y}"
                );
                let mut dso = pantocaren
                    .iter()
                    .filter_map(|&(angle_deg, lever)| {
                        let angle_rad: f64 = angle_deg.to_radians();
                        let v1 = lever;
                        let v2 = z_g_fix * angle_rad.sin();
                        let v3 = delta_y * angle_rad.cos();
                        let value = v1 - v2 - v3;
                        //    if angle_deg.fract() == 0. {
                        //        log::info!("{}", format!("LeverDiagram calculate расчет диаграммы: theta deg:{angle_deg}, l_k:{v1}, z_g_fix*sin(theta):{v2}, (y_g - y_c)*cos(theta):{v3}, l:{value}"));
                        //    }
                        Some((angle_deg, value))
                    })
                    .collect::<Vec<(f64, f64)>>();
                // плечо для нулевого угла
                let lever_zero = dso
                    .iter()
                    .find(|(a, _)| *a == 0.)
                    .ok_or(error.err("calculate lever_zero error!"))?.1;
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
                let dso_curve = Curve::new_linear(&dso).map_err(|e| error.pass_with("calculate curve", e))?;
                let mut angle = tmp_dso
                    .first()
                    .expect("LeverDiagram calculate error, no dso values!")
                    .0;
                let mut theta_max = angle;
                let mut value = dso_curve.value(angle).map_err(|e| error.pass_with("calculate value", e))?;
                let mut max_value = value;
                let mut delta_angle = 1.;
                for _i in 0..10 {
                    let delta_angle_l = angle - delta_angle;
                    let value_l = dso_curve.value(delta_angle_l).map_err(|e| error.pass_with("calculate value_l", e))?;
                    let delta_angle_r = angle + delta_angle;
                    let value_r = dso_curve.value(delta_angle_r).map_err(|e| error.pass_with("calculate value_r", e))?;
                    if value_l >= value_r {
                        value = value_l;
                        angle -= delta_angle;
                    } else {
                        value = value_r;
                        angle += delta_angle;
                    }
                    if value >= max_value {
                        max_value = value;
                        theta_max = angle;
                    } else {
                        angle = theta_max;
                    }
                    delta_angle *= 0.5;
                    //    log::info!("{}", format!("LeverDiagram calculate max_angle: value:{value} angle:{angle} max_value:{max_value} max_angle:{max_angle} delta_angle:{delta_angle} i:{_i} "));
                }
                log::trace!(
                    "{}",
                    format!("LeverDiagram calculate max_angle:{theta_max}")
                );
                // нахождение углов максимумов и угла пересечения с 0
                let mut max_angles: Vec<(f64, f64)> = Vec::new();
                let mut last_value = dso_curve.value(0.).map_err(|e| error.pass_with("calculate last_value", e))?;
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
                        dso_curve.value(theta_max).map_err(|e| error.pass_with("calculate max_angles", e))?
                    ));
                }
                //
                let angle_zero = crate::algorithm::eval::lever_diagram_eval::lever_diagram_ctx::angle(theta_max, &dso_curve,  0.);
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
                /*   log::trace!(
                    "LeverDiagram calculate z_g_fix:{z_g_fix} angle_zero:{}",
                    angle_zero * angle_zero_signum,
                );*/
                log::trace!("LeverDiagram calculate diagram: [angle dso ddo]:");
                for &(angle, dso, ddo) in diagram.iter() {
                    log::trace!("{angle} {dso} {ddo};");
                }
                ctx.write_params(ParameterID::Roll, angle_zero * angle_zero_signum);
                let result = LeverDiagramCtx {
                    dso,
                    dso_curve,
                    ddo,
                    diagram,
                    theta_max,
                    max_angles,
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
