use sal_sync::services::entity::error::str_err::StrErr;
use crate::{
    algorithm::context::context_access::ContextReadRef, kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::ModelLink, ContextWrite, CtxResult
};
use super::areas_strength_ctx::AreasStrengthCtx;
///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct AreasStrengthEval {
    dbg: DbgId,
    model: ModelLink,
    value: Option<AreasStrengthCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl AreasStrengthEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(parent: impl Into<String>, model: ModelLink, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "AreasStrength");
        Self {
            dbg,
            model,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for AreasStrengthEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let unit = match initial.unit.clone() {
                        Some(data) => data.data(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read unit error: no data!",
                                self.dbg
                            )))
                        }
                    };                    
                    let (area_const_v, area_const_h) = match self.model.areas().await {
                        Ok((area_const_v, area_const_h)) => {
                            (area_const_v, area_const_h)
                        },                        
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read areas error: {:?}",
                                self.dbg, err
                            )));
                        },
                    };
                    // Ищем площадь горизонтальной поверхности палубных грузов.
                    // Перебираем горизонтальую поверхность с шагом, проходим по грузам и
                    // берем максимальную площадь среди всех грузов на этом шаге.
                    let min_x = unit
                        .iter()
                        .filter_map(|v| v.min_x())
                        .min_by(|&a, &b| a.partial_cmp(&b).unwrap());
                    let max_x = unit
                        .iter()
                        .filter_map(|v| v.max_x())
                        .max_by(|&a, &b| a.partial_cmp(&b).unwrap());


                    let area_v = area_const_v.
                    /// Площадь горизонтальных поверхностей открытых палуб
                    let area_h = 




                    let area_timber_h = 
                    let mut area_sum = 0.;
                    for v in self.desks.iter().filter(|v| v.is_timber()) {
                        area_sum += v.horizontal_area(
                            &bound_x.intersect(&self.icing_timber_bound.bound_x()?)?,
                            &self.icing_timber_bound.bound_y()?,
                        )?;
                    }
                    Ok(area_sum)


                    self.value = Some(result.clone());
                    ctx.write(result)                    
                },
                CtxResult::Err(err) => CtxResult::Err(StrErr(format!(
                    "{}.eval | Read context error: {:?}",
                    self.dbg, err
                ))),
                CtxResult::None => CtxResult::None,
            }
        })
    }
}
//
//
impl std::fmt::Debug for AreasStrengthEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AreasStrength")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}