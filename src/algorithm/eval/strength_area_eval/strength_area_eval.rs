use super::strength_area_ctx::StrengthAreaCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{data::loads::UnitCargoType, Bound},
        eval::IcingTimberCtx,
    },
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ship_model::model_link::ModelLink,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;
///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct StrengthAreaEval {
    dbg: DbgId,
    model: ModelLink,
    value: Option<StrengthAreaCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl StrengthAreaEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        model: ModelLink,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "BoundArea");
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
impl Eval<(), EvalResult> for StrengthAreaEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let unit: Vec<_> = match initial.unit.as_ref() {
                        Some(data) => data.data().into_iter().filter(|v| v.icing_area.is_some()).collect(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read unit error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let timber_unit: Vec<_> = unit.iter().filter(|v| v.cargo_type == UnitCargoType::Timber).collect();
                    let bounds = match initial.bounds.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bounds error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let (const_area_v, const_area_h) = match self.model.bound_areas().await {
                        Ok((area_v, area_h)) => (area_v, area_h),
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bound_areas error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };
                    let icing_timber_bound: IcingTimberCtx = ctx.read();
                    let icing_timber_bound_x = match icing_timber_bound.bound_x() {
                        Ok(data) => data,
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_timber_bound_x error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };
                    let icing_timber_bound_y = match icing_timber_bound.bound_y() {
                        Ok(data) => data,
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_timber_bound_y error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };
                    // Ищем площадь парусности палубных грузов.
                    // Перебираем поверхность парусности с шагом, проходим по грузам и
                    // берем площадь как диапазон между максимальными ограничениями всех грузов на этом шаге.                    
                    let mut area_v = Vec::new();
                    // Границы грузов
                    let min_x = unit.iter()
                        .filter_map(|v| v.bound_x1)
                        .min_by(|&a, &b| a.partial_cmp(&b).unwrap());
                    let max_x = unit.iter()
                        .filter_map(|v| v.bound_x2)
                        .max_by(|&a, &b| a.partial_cmp(&b).unwrap());
                    // Если есть границы грузов ищемраспределения площадей грузов
                    if let (Some(min_x), Some(max_x)) = (min_x, max_x) { 
                        // Диапазон грузов по оси Х
                        let units_bound = match Bound::new(min_x, max_x) {
                            Ok(data) => data,
                            Err(err) => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | units_bound error: {:?}",
                                    self.dbg, err
                                )));
                            }
                        };
                        // Перебираем шпации и ищем площадь попавшую в текущую шпацию
                        for (i, bound_x) in bounds.iter().enumerate() { 
                            // Площадь парусности корпуса, попадающая в текущую шпацию
                            let mut current_area = match const_area_v.get(i) {
                                Some(&data) => data,
                                None => {
                                    return CtxResult::Err(StrErr(format!(
                                        "{}.eval | const_area_h.get error: no value for bound {i}", self.dbg
                                    )));
                                }
                            };
                            // Пересечение шпации и диапазона грузов
                            let bound_x = match bound_x.intersect(&units_bound) { 
                                Ok(data) => data,
                                Err(err) => {
                                    return CtxResult::Err(StrErr(format!(
                                        "{}.eval | bound_x.intersect error: {:?}",
                                        self.dbg, err
                                    )));
                                }
                            };
                            // Если есть пересечение шпации и диапазона грузов
                            if bound_x.is_some() {     
                                // грузы имеющие площадь парусности в текущей шпации
                                let unit = unit.iter()
                                    .filter(|v| v.windage_area(&bound_x, &Bound::Full).unwrap_or(0.) > 0. );
                                // границы грузов в текущей шпации
                                let min_z = unit.clone()
                                    .filter_map(|v| v.bound_z1 )
                                    .min_by(|&a, &b| a.partial_cmp(&b).unwrap());
                                let max_z = unit
                                    .filter_map(|v| v.bound_z2)
                                    .max_by(|&a, &b| a.partial_cmp(&b).unwrap());
                                // Прибавляем к площади прямоугольник площади грузов
                                if let (Some(min_z), Some(max_z)) = (min_z, max_z) { 
                                    current_area += bound_x.length().unwrap_or(0.) * (max_z - min_z);
                                }
                            }
                            area_v.push(current_area);
                        }
                    }
                    // Горизонтальная площадь палубного груза - леса
                    let mut area_timber_h_values = Vec::new();
                    for bound_x in bounds.iter() {
                        let mut area = 0.;
                        for u in &timber_unit {
                            area += match u.icing_area(
                                &bound_x.intersect(&icing_timber_bound_x).unwrap_or(Bound::None),
                                &icing_timber_bound_y,
                            ) {
                                Ok(area) => area,
                                Err(err) => {
                                    return CtxResult::Err(StrErr(format!(
                                        "{}.eval | Read unit horizontal_area error: {:?}",
                                        self.dbg, err
                                    )))
                                }
                            };
                        }
                        area_timber_h_values.push(area);
                    }     
                    let result = StrengthAreaCtx {
                        area_v,
                        area_v_shift,
                        area_v_values,
                        area_h,
                        area_h_shift,
                        area_h_values,
                        area_timber_h: area_timber_h_values.iter().sum(),
                        area_timber_h_shift,
                        area_timber_h_values,
                    };
                    self.value = Some(result.clone());
                    ctx.write(result)
                }
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
impl std::fmt::Debug for StrengthAreaEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundArea")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
