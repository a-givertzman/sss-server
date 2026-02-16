use std::{borrow::Borrow, fmt::Debug, hash::Hash};
use sal_core::error::Error;
use crate::{kernel::{EvalEx, sync::Link, types::fx_map::FxIndexMap}, server::{Cot, ErrorCode, ErrorReply, EvalResult, Request}};
///
/// Matching incoming messages by it's Cot
/// - Forwarding matched messages to the associated handlers
pub struct SelectCot<K> {
    select: FxIndexMap<Cot, Box<dyn EvalEx<(Request<K>, Option<Link>), EvalResult<K>> + Send>>,
}
//
//
impl<K> SelectCot<K> {
    ///
    /// Returns [SortByX] new instance
    pub fn new(select: Vec<(Cot, Box<dyn EvalEx<(Request<K>, Option<Link>), EvalResult<K>> + Send + 'static>)>) -> Self {
        Self {
            select: FxIndexMap::from_iter(select),
        }
    }
}
//
//
impl<K: Borrow<K> + Hash + Eq + Debug + Copy> EvalEx<(Request<K>, Option<Link>), EvalResult<K>> for SelectCot<K> {
    //
    //
    fn eval(&self, (request, link): (Request<K>, Option<Link>)) -> EvalResult<K> {
        let error = Error::new("SelectCot", "eval");
        match self.select.get(&request.cot) {
            Some(eval) => {
                match request.cot {
                    Cot::Act => eval.eval((request, link)),
                    Cot::Req => eval.eval((request, None)),
                    _ => Ok(Some(request.reply_err(ErrorReply::new(
                        ErrorCode::BadRequest,
                        error.err(format!("Cot '{:?}' - is not supported", request.cot)),
                    )))),
                }
            },
            None => Ok(Some(request.reply_err(ErrorReply::new(
                ErrorCode::BadRequest,
                error.err(format!("Cot '{:?}' - is not supported", request.cot)),
            )))),
        }
    }
    ///
    /// Halts all configured hanblers
    fn exit(&self) {
        for (_, sel) in &self.select {
            sel.exit();
        }
    }
}
