use super::DraftMarkResult;
use crate::algorithm::context::context_access::ContextParamsWrite;
use sal_3dlib_core::math::*;
use crate::algorithm::entities::draught::Draught;
use crate::algorithm::eval::{DraftMarkCtx, parameters::ParameterID};
use crate::{
    algorithm::context::context_access::ContextReadRef,
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::ContextWrite,
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет уровня заглубления для координат отметок заглубления на корпусе судна
pub struct DraftMarkEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl DraftMarkEval {
    //
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "DraftMarkEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for DraftMarkEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let data = initial.draft_mark.clone().unwrap();
                let mut result = Vec::new();
                let draught = Draught::new(&self.dbg, &ctx).map_err(|err| error.pass(err))?;
                for p in data.iter() {
                    if p.data.len() <= 2 {
                        log::error!("DraftMark eval error: p.data.len() <= 2, {}", p.name);
                        continue;
                    }
                    let mut z_fix: Vec<(f64, f64, f64, f64)> = Vec::new();
                    for v in p.data.iter() {
                        z_fix.push((
                            v.x(),
                            v.y(),
                            v.z(),
                            v.z() - draught.value(v),
                        ));
                    }
                    z_fix.sort_by(|a, b| {
                        a.3.abs()
                            .partial_cmp(&b.3.abs())
                            .expect("DraftMark calculate error: partial_cmp!")
                    });
                    // Если все марки ниже или выше уровня воды
                    if z_fix[0].3.signum() == z_fix[1].3.signum() {
                        if z_fix[0].3.abs() < 0.001 {
                            // если уровень прямо на марке
                            result.push(DraftMarkResult::new(
                                p.criterion_id,
                                p.name.clone(),
                                z_fix[0].0,
                                z_fix[0].1,
                                Some(z_fix[0].2),
                            ));
                            ctx.write_params(ParameterID::from(p.criterion_id)?, z_fix[0].2);
                        } else {
                            result.push(DraftMarkResult::new(
                                p.criterion_id,
                                p.name.clone(),
                                z_fix[0].0,
                                z_fix[0].1,
                                None,
                            ));
                        }
                        continue;
                    }
                    // Интерполированные значения координат марок заглубления
                    let fix_x = Curve::new_linear(
                        &z_fix.iter().map(|&v| (v.3, v.0)).collect::<Vec<_>>()[..],
                    )
                    .map_err(|err| error.pass_with("fix_x", err))?
                    .value(0.)
                    .map_err(|err| error.pass_with("fix_x", err))?;
                    let fix_y = Curve::new_linear(
                        &z_fix.iter().map(|&v| (v.3, v.1)).collect::<Vec<_>>()[..],
                    )
                    .map_err(|err| error.pass_with("fix_y", err))?
                    .value(0.)
                    .map_err(|err| error.pass_with("fix_y", err))?;
                    let fix_z = Curve::new_linear(
                        &z_fix.iter().map(|&v| (v.3, v.2)).collect::<Vec<_>>()[..],
                    )
                    .map_err(|err| error.pass_with("fix_z", err))?
                    .value(0.)
                    .map_err(|err| error.pass_with("fix_z", err))?;
                    result.push(DraftMarkResult::new(
                        p.criterion_id,
                        p.name.clone(),
                        fix_x,
                        fix_y,
                        Some(fix_z),
                    ));
                    ctx.write_params(ParameterID::from(p.criterion_id)?, fix_z);
                }
                log::info!(
                    "DraftMark result:{}\n",
                    result.iter().fold(String::new(), |s, v| s + &format!(
                        "\n{} {} ({:.3} {:.3} {:?})",
                        v.criterion_id,
                        v.name,
                        v.x,
                        v.y,
                        v.z
                    ))
                );
                let result = DraftMarkCtx { data: result };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DraftMarkEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DraftMarkEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
