use crate::algorithm::entities::data::loads::UnitCargoType;
use crate::algorithm::entities::data::stability::ship_type::*;
use crate::algorithm::eval::criterion::*;
use crate::algorithm::eval::stability::*;
use crate::algorithm::eval::zg::Zg;
use crate::{
    prelude::*,
    kernel::{Eval, types::eval_result::EvalResult},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Статический угол крена от действия постоянного ветра.
pub struct StaticAngleEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl StaticAngleEval {
    //
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "StaticAngleEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for StaticAngleEval {
    /// Статический угол крена от действия постоянного ветра.
    /// При расчете плеча кренящего момента от давления ветра 𝑙𝑤1, используемое при
    /// определении угла крена θ𝑤1, предполагаемое давление ветра 𝑝𝑣 принимается как для судна
    /// неограниченного района плавания судна.
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_type = initial.ship_type.unwrap();
                let have_container = initial.unit.as_ref()
                    .ok_or(error.err("initial.unit no data"))?
                    .iter()
                    .any(|v| v.cargo_type == UnitCargoType::Container);
                let wind: WindCtx = ctx.read();
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let flooding_angle = lever_diagram.flooding_angle;
                // Для всех судов (кроме района плавания R3):
                // статического угла крена θ𝑤1, вызванного постоянным ветром
                let wind_lever = wind.arm_wind_static;
                let angles = match lever_diagram.angle(wind_lever) {
                    Ok(angles) => angles,
                    Err(error) => {
                        let error = error.pass_with("lever_diagram.angle", error.clone());
                        log::error!("{}", error);      
                        let result = StaticAngleCtx { 
                            data: CriterionData::new_error(
                                CriterionID::WindStaticHeel,
                                "Ошибка расчета угла крена судна соответствующего плечу кренящего момента постоянного ветра: ".to_owned() + &error.to_string(),
                            )
                        };
                        return ctx.write(result);
                    }
                };
                let angle = angles.first();
                let target_value = if ship_type == ShipType::TimberCarrier {
                    16.
                } else if have_container {
                    16.0f64.min(0.5 * flooding_angle)
                } else {
                    16.0f64.min(0.8 * flooding_angle)
                };
                let data = if let Some(angle) = angle {
                    log::info!(
                        "Criterion WindStaticHeel result:{:.3} target:{:.3} ",
                        angle,
                        target_value
                    ); 
                    CriterionData::new_result(CriterionID::WindStaticHeel, *angle, target_value)
                } else {
                    log::error!(
                        "Criterion WindStaticHeel: Нет угла крена судна для текущих погодных условий"
                    ); 
                    CriterionData::new_error(
                        CriterionID::WindStaticHeel,
                        "Нет угла крена судна для текущих погодных условий".to_owned(),
                    )
                };
                let result = StaticAngleCtx { 
                    data 
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for StaticAngleEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticAngleEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
