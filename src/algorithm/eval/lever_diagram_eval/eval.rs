use crate::algorithm::eval::LeverDiagramCtx;
use crate::kernel::Eval;
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextRead},
        entities::math::curve::*,
        eval::{parameters::ParameterID, zg_eval::Zg, StabilityBalanceCtx},
    }, kernel::{types::eval_result::EvalResult}, prelude::ContextWrite,
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Диаграмма плеч статической и динамической остойчивости
pub struct LeverDiagramEval {
    dbg: Dbg,
  //  model: Link,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl LeverDiagramEval {
    ///
    pub fn new(
        parent: impl Into<String>,
  //      model: Link,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "LeverDiagramEval");
        Self {
            dbg,
    //        model,
            ctx: Box::new(ctx), 
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for LeverDiagramEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
        //        let ctx = self.ctx.take().unwrap();
                let balance: StabilityBalanceCtx = ctx.read();
                let dso = balance.dso; 
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
                let angle_zero = crate::algorithm::eval::lever_diagram_eval::ctx::angle(theta_max, &dso_curve,  0.);
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
                let result = LeverDiagramCtx {
                    dso,
                    dso_curve,
                    ddo,
                    diagram,
                    theta_max,
                    max_angles,
                };
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
