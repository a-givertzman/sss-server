use std::{borrow::Borrow, fmt::Debug, hash::Hash};
use sal_core::error::Error;
use crate::{kernel::{EvalEx, sync::Link, types::fx_map::FxIndexMap}, server::{ErrorCode, ErrorReply, EvalResult, Request}};

///
/// Matching incoming messages by it's Cot::Req name
/// - Forwarding matched messages to the associated handlers
/// - Returns bytes and id of messages to be sent over TCP
pub struct SelectAct<K> {
    select: FxIndexMap<K, Box<dyn EvalEx<(Request<K>, Option<Link>), Result<(), Error>> + Send>>,
}
//
//
impl<K: Hash + Eq> SelectAct<K> {
    ///
    /// Returns [SelectAct] new instance
    pub fn new(select: Vec<(K, Box<dyn EvalEx<(Request<K>, Option<Link>), Result<(), Error>> + Send + 'static>)>) -> Self {
        Self {
            select: FxIndexMap::from_iter(select),
        }
    }
}
//
//
impl<K: Borrow<K> + Hash + Eq + Debug + Copy> EvalEx<(Request<K>, Option<Link>), EvalResult<K>> for SelectAct<K> {
    //
    //
    fn eval(&self, (request, link): (Request<K>, Option<Link>)) -> EvalResult<K> {
        let error = Error::new("SelectAct", "eval");
        match self.select.get(&request.query_id) {
            Some(eval) => {
                let query_id = request.query_id;
                let response = request.reply_empty();
                match eval.eval((request, link)) {
                    Ok(_) => Ok(None),
                    Err(err) => Ok(Some(response.into_err(ErrorReply::new(
                        ErrorCode::InternalError,
                        error.pass_with(format!("Request '{:?}' - failed", query_id), err),
                    )))),
                }
            },
            None => Ok(Some(request.reply_err(ErrorReply::new(
                ErrorCode::BadRequest,
                error.err(format!("Request '{:?}' - is not supported", request.cot)),
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
//
//
unsafe impl<Q> Send for SelectAct<Q> {}
