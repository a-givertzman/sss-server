use super::static_angle_ctx::StaticAngleCtx;
use crate::algorithm::entities::data::stability::{multipler_s::*, ship_type::*, *};
use crate::algorithm::entities::math::curve::*;
use crate::{
    ContextWrite, CtxResult,
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        eval::{
            LeverDiagramCtx, MetacentricHeightCtx, RollingAmplitudeCtx, RollingPeriodCtx, WindCtx,
        },
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ship_model::model_link::{IModelLink, ModelLink},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Статический угол крена от действия постоянного ветра.
pub struct StaticAngleEval {
    dbg: Dbg,
    value: Option<StaticAngleCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl StaticAngleEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "StaticAngleEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for StaticAngleEval {
    /// Статический угол крена от действия постоянного ветра.
    /// При расчете плеча кренящего момента от давления ветра 𝑙𝑤1, используемое при
    /// определении угла крена θ𝑤1, предполагаемое давление ветра 𝑝𝑣 принимается как для судна
    /// неограниченного района плавания судна.
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let have_container = match initial.unit.as_ref() {
                    Some(data) => data
                        .into_iter()
                        .any(|v| v.cargo_type == UnitCargoType::Container),
                    None => return CtxResult::Err(error.err("Read unit error: no data!")),
                };
                let parameters: Parameters = ctx.read();
                let wind: WindCtx = ctx.read();
                let lever_diagram: LeverDiagramCtx = ctx.read();
                let balance: BalanceCtx = ctx.read();
                let flooding_angle = balance.flooding_angle;
                // Для всех судов (кроме района плавания R3):
                // статического угла крена θ𝑤1, вызванного постоянным ветром
                let wind_lever = wind.arm_wind_static;
                let angles = match lever_diagram.angle(wind_lever) {
                    Ok(angles) => angles,
                    Err(error) => {
                        let error = CtxResult::Err(error.pass_with("lever_diagram.angle", error));
                        log::error!("{error}");
                        return CriterionData::new_error(
                            CriterionID::WindStaticHeel,
                            "Ошибка расчета угла крена судна соответствующего плечу кренящего момента постоянного ветра: ".to_owned() + &error.to_string(),
                        );
                    }
                };
                let angle = angles.first();
                let ship_type = ShipType::from_str(
                    &initial
                        .ship
                        .expect("static_angle eval error: no ship!")
                        .ship_type,
                )?;
                let target_value = if ship_type == ShipType::TimberCarrier {
                    16.
                } else if have_container {
                    16.0f64.min(0.5 * flooding_angle)
                } else {
                    16.0f64.min(0.8 * flooding_angle)
                };
                let data = if let Some(angle) = angle {
                    CriterionData::new_result(CriterionID::WindStaticHeel, *angle, target_value)
                } else {
                    CriterionData::new_error(
                        CriterionID::WindStaticHeel,
                        "Нет угла крена судна для текущих погодных условий".to_owned(),
                    )
                };
                let result = StaticAngleCtx { 
                    data 
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
impl std::fmt::Debug for WindEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
