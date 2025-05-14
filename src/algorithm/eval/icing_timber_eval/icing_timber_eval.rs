use sal_core::{dbg::Dbg, error::Error};
use crate::{
    algorithm::context::context_access::ContextReadRef, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite
};
use super::icing_timber_ctx::{IcingTimberCtx, IcingTimberType};

///
/// Ограничение горизонтальной площади обледенения палубного груза - леса
pub struct IcingTimberEval {
    dbg: Dbg,
    value: Option<IcingTimberCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl IcingTimberEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "IcingTimberEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for IcingTimberEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let voyage = initial.voyage.as_ref().ok_or(error.err("voyage error: no data!"))?; 
                let icing_timber_stab = IcingTimberType::from_str(&voyage.icing_timber_type)
                    .map_err(|err| error.pass_with("icing_timber_stab", err))?;
                let ship_parameters = initial.ship_parameters.as_ref().ok_or(error.err("ship_parameters error: no data!"))?; 
                let length_loa = *ship_parameters.get("L.O.A").ok_or(error.err("length_loa error: no data!"))?; 
                let width = *ship_parameters.get("MouldedBreadth").ok_or(error.err("width error: no data!"))?;                
                let result = IcingTimberCtx::new(width, length_loa, icing_timber_stab);
                self.value = Some(result.clone());
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
        f.debug_struct("IcingTimberEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
