use crate::algorithm::entities::Moment;
use crate::algorithm::eval::icing_timber::ctx::IcingTimberCtx;
use crate::algorithm::eval::icing_timber_bound::ctx::IcingTimberBoundCtx;
use crate::kernel::Eval;
use crate::prelude::ContextRead;
use crate::{
    algorithm::entities::{Bound, data::loads::UnitCargoType},
    kernel::types::eval_result::EvalResult,
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет площади обледенения горизонтальных поверхностей палубного лесного груза
pub struct IcingTimberEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl IcingTimberEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingTimberEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for IcingTimberEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let unit: Vec<_> = match initial.unit.as_ref() {
                    Some(data) => data
                        .iter()
                        .filter(|v| v.icing_area.is_some())
                        .collect(),
                    None => return Err(error.err("Read unit error: no data!")),
                };
                let timber_unit: Vec<_> = unit
                    .iter()
                    .filter(|v| v.cargo_type == UnitCargoType::Timber)
                    .collect();
                let bounds = match initial.bounds.clone() {
                    Some(data) => data,
                    None => return Err(error.err("Read bounds error: no data!")),
                };
                let icing_timber_bound: IcingTimberBoundCtx = ctx.read();
                let icing_timber_bound_x = match icing_timber_bound.bound_x() {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(error.pass_with("Read icing_timber_bound_x error", err));
                    }
                };
                let icing_timber_bound_y = match icing_timber_bound.bound_y() {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(error.pass_with("Read icing_timber_bound_y error", err));
                    }
                };
                let mut area_icing = 0.;
                let mut area_full = 0.;
                let mut moment_icing = Moment::zero();
                let mut moment_full = Moment::zero();
                let mut delta_moment_icing = Moment::zero();
                for u in &timber_unit {
                    let (icing_current_area, icing_moment, icing_delta_moment) = u
                        .icing_area(&icing_timber_bound_x, &icing_timber_bound_y)
                        .map_err(|err| error.pass(err))?;
                    let (full_current_area, full_current_moment, _) = u
                        .icing_area(&Bound::Full, &Bound::Full)
                        .map_err(|err| error.pass(err))?;
                    area_icing += icing_current_area;
                    area_full += full_current_area;
                    moment_icing += icing_moment;
                    moment_full += full_current_moment;
                    delta_moment_icing += icing_delta_moment;
                }
                let mut area_icing_distr = Vec::new();
                let mut area_no_icing_distr = Vec::new();
                for bound_x in bounds.iter() {
                    let mut icing_current_area = 0.;
                    let mut full_current_area = 0.;
                    for u in &timber_unit {
                        match u.icing_area(
                            &bound_x
                                .intersect(&icing_timber_bound_x)
                                .unwrap_or(Bound::None),
                            &icing_timber_bound_y,
                        ) {
                            Ok((area, _, _)) => {
                                icing_current_area += area;
                            }
                            Err(err) => {
                                return Err(error.pass_with("Read unit horizontal_area error", err));
                            }
                        };
                        match u.icing_area(bound_x, &Bound::Full) {
                            Ok((area, _, _)) => {
                                full_current_area += area;
                            }
                            Err(err) => {
                                return Err(error.pass_with("Read unit horizontal_area error", err));
                            }
                        };
                    }
                    area_icing_distr.push(icing_current_area);
                    area_no_icing_distr.push(full_current_area - icing_current_area);
                }
                let result = IcingTimberCtx {
                    area_icing,
                    area_no_icing: area_full - area_icing,
                    moment_icing,
                    moment_no_icing: moment_full - moment_icing,
                    delta_moment_icing,
                    area_icing_distr,
                    area_no_icing_distr,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for IcingTimberEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcingTimberCtx")
            .field("dbg", &self.dbg)
            .finish()
    }
}
