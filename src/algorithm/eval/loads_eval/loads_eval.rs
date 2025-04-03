use super::loads_ctx::LoadsCtx;
use crate::algorithm::context::context_access::*;
use crate::algorithm::entities::data::loads::UnitCargoType;
use crate::algorithm::entities::{Moment, Position};
use crate::{
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
pub struct LoadsEval {
    dbg: DbgId,
    value: Option<LoadsCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl LoadsEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "LoadsEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for LoadsEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let shift_const = if let Some(ship_parameters) =
                        initial.ship_parameters.as_ref()
                    {
                        let const_mass_shift_x = *match ship_parameters.get("LCG from middle") {
                            Some(data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | Read const_mass_shift_x error: no data!",
                                    self.dbg
                                )))
                            }
                        };
                        let const_mass_shift_y = *match ship_parameters.get("TCG from CL") {
                            Some(data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | Read const_mass_shift_y error: no data!",
                                    self.dbg
                                )))
                            }
                        };
                        let const_mass_shift_z = *match ship_parameters.get("VCG from BL") {
                            Some(data) => data,
                            None => {
                                return CtxResult::Err(StrErr(format!(
                                    "{}.eval | Read const_mass_shift_z error: no data!",
                                    self.dbg
                                )))
                            }
                        };
                        Position::new(const_mass_shift_x, const_mass_shift_y, const_mass_shift_z)
                    } else {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Read const_mass_shift_z error: no ship_parameters!",
                            self.dbg
                        )));
                    };
                    let mass_const = match initial.load_constant.clone() {
                        Some(data) => data.data().iter().map(|v| v.mass).sum(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read load_constant error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let bulk: Vec<_> = match initial.bulk.clone() {
                        Some(data) => data.data().iter().map(|v| v.data()).collect(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bulk error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let liquid: Vec<_> = match initial.liquid.clone() {
                        Some(data) => data.data().iter().map(|v| v.data()).collect(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read liquid error: no data!",
                                self.dbg
                            )))
                        }
                    };

                    let (mass_unit, shift_unit, grain_bulkhead) = match initial.unit.clone() {
                        Some(data) => {
                            let unit = data.data();
                            let grain_bulkhead: Vec<_> = unit
                                .iter()
                                .filter(|v| {
                                    v.cargo_type == UnitCargoType::GrainBulkhead
                                        && v.bound_x().is_ok()
                                })
                                .map(|v| v.bound_x().unwrap().center())
                                .flatten()
                                .collect();
                            let (mass_unit, shift_unit) = unit
                                .iter()
                                .filter_map(|v| match v.mass_shift() {
                                    Ok(mass_shift) => Some((v.mass, mass_shift)),
                                    Err(_) => None,
                                })
                                .fold(
                                    (0., Moment::zero()),
                                    |(mass_sum, moment_sum), (mass, mass_shift)| {
                                        (
                                            mass_sum + mass,
                                            moment_sum + Moment::from_pos(mass_shift, mass),
                                        )
                                    },
                                );
                            (mass_unit, shift_unit, grain_bulkhead)
                        }
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read unit error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let (mass_gaseous, shift_gaseous) = match initial.gaseous.clone() {
                        Some(data) => data
                            .data()
                            .iter()
                            .filter_map(|v| match v.mass_shift {
                                Some(mass_shift) => Some((v.mass, mass_shift)),
                                None => None,
                            })
                            .fold(
                                (0., Moment::zero()),
                                |(mass_sum, moment_sum), (mass, mass_shift)| {
                                    (
                                        mass_sum + mass,
                                        moment_sum + Moment::from_pos(mass_shift, mass),
                                    )
                                },
                            ),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read gaseous error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let result = LoadsCtx {
                        mass_const,
                        mass_bulk: bulk.iter().fold(0., |sum, v| sum + v.mass),
                        mass_liquid: liquid.iter().fold(0., |sum, v| sum + v.mass),
                        mass_unit,
                        mass_gaseous,
                        shift_const,
                        shift_unit,
                        shift_gaseous,
                        bulk,
                        liquid,
                        grain_bulkhead,
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
impl std::fmt::Debug for LoadsEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadsEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
