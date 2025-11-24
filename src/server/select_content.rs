use std::{borrow::Borrow, fmt::Debug, hash::Hash};
use sal_core::error::Error;
use sal_sync::collections::FxIndexMap;
use crate::{kernel::{EvalEx, sync::Link}, server::{Content, EvalResult, Event, Request}};
///
/// Matching incoming [Event]s by it's Content
/// - Forwarding matched [Event]s to the associated handlers
pub struct SelectContent<K> {
    select: FxIndexMap<Content, Box<dyn EvalEx<(Request<K>, Option<Link>), EvalResult<K>> + Send>>,
}
//
//
impl<K> SelectContent<K> {
    ///
    /// Returns [SelectContent] new instance
    pub fn new(
        select: Vec<(Content, Box<dyn EvalEx<(Request<K>, Option<Link>), EvalResult<K>> + Send + 'static>)>,
    ) -> Self {
        Self {
            select: FxIndexMap::from_iter(select),
        }
    }
}
//
//
impl<K: Borrow<K> + Hash + Eq + Debug + Copy> EvalEx<(Event<K>, Option<Link>), EvalResult<K>> for SelectContent<K> {
    ///
    /// Selects handler by [Event] content type,
    /// if handler exists, it evaluates with [Request] built from [Event]
    fn eval(&self, (event, link): (Event<K>, Option<Link>)) -> EvalResult<K> {
        let error = Error::new("SelectContent", "eval");
        match self.select.get(&event.content) {
            Some(eval) => match Request::from_event(event) {
                Ok(req) => eval.eval((req, link)),
                Err(err) => Err(error.pass(err)),
            },
            None => Err(error.err(format!("{:?} - is not supported", event.content))),
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
