
use sal_core::{dbg::Dbg, error::Error};

use crate::algorithm::entities::data::loads::*;
use crate::algorithm::entities::Bounds;
use crate::prelude::InitialCtx;
use crate::{
    algorithm::context::{
            context::Context,
            context_access::{ContextReadRef, ContextWrite},
        },
    kernel::{eval::Eval, types::eval_result::EvalResult},
};

use super::data::*;


///
/// Заглушка для тестирования, имитирует ввод данных. Содержит все данные
/// для расчетов.
#[derive(Debug)]
pub struct FakeInitial {
    dbg: Dbg,
    ctx: Context,
}
//
//
impl FakeInitial {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: Context,
    ) -> Self {
        let dbg = Dbg::new(parent, "FakeInitial");
        Self {
            dbg,
            ctx,
        }
    }
}
//
impl Eval<(), EvalResult> for FakeInitial {
    fn eval(&mut self, _: ()) -> EvalResult {
        let _error = Error::new(&self.dbg, "eval");
        let initial_ctx: &InitialCtx = self.ctx.read_ref();
        let mut initial_ctx = initial_ctx.to_owned();
        // Расчет баланса в модели
        let bounds = Bounds::from_min_max(-3.6, 135.5, 200).unwrap();
        let ship = ship();
        let ship_parameters = ship_parameters();
        let voyage = voyage();
        let icing = icing();
        let load_constant = load_constant();
        let bulk = Vec::new();
        let gaseous = gaseous(); 
        let unit = Vec::new();
        let liquid = liquid();            
        initial_ctx.bounds = Some(bounds);
        initial_ctx.ship = Some(ship);
        initial_ctx.ship_parameters = Some(ship_parameters);
        initial_ctx.voyage = Some(voyage);
        initial_ctx.icing = Some(icing);
        initial_ctx.load_constant = Some(load_constant);
        initial_ctx.bulk = Some(bulk);
        initial_ctx.liquid = Some(liquid);
        initial_ctx.unit = Some(unit);
        initial_ctx.gaseous = Some(gaseous);
        self.ctx.clone().write(initial_ctx.to_owned())
    }
}
