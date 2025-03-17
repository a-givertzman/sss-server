//! Учет намокания груза
use crate::algorithm::context::context_access::*;
use crate::algorithm::entities::{Moment, Position};
use crate::{
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;

use super::wetting_ctx::WettingCtx;

///
/// Учет намокания палубного груза.  
/// При расчете намокания необходимо учитывать изменения водоизмещения и  
/// возвышения центра тяжести. Масса намокания и его моменты учитывается
/// при расчете прочности.
pub struct WettingEval {
    dbg: DbgId,
    value: Option<WettingCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl WettingEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "WettingEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for WettingEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let voyage = match initial.voyage.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read voyage error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let dry = match initial.dry.clone() {
                        Some(data) => data.data(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read Wetting error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let (mass, mass_moment) = dry.into_iter().fold((0., Moment::zero()), |(mass, moment), v| {
                        match (v.mass, v.mass_shift, v.permeability)  {
                            (Some(mass), Some(mass_shift), Some(permeability)) => (mass*permeability, Moment::from_pos(mass_shift, mass*permeability)),
                            _ => (0., Position::zero()),
                        }
                    });
                    let mass_shift = mass_moment.scale(1./mass);
                    let result = WettingCtx {
                        mass,
                        mass_shift,
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
impl std::fmt::Debug for WettingEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("WettingEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
