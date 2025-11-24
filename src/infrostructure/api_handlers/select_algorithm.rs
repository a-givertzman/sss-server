use std::fmt::Debug;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::Scheduler;
use crate::{conf::AlgorithmConf, kernel::{Eval, EvalEx, sync::Link, types::eval_result::EvalResult}, server::{self, AlgorithmQuery, AlgorithmReply, Query, Reply, Request, extract}};

///
/// Evaluates entair ship calculations in the separate thread
pub struct SelectAlgorithm {
    dbg: Dbg,
    conf: AlgorithmConf,
    scheduler: Scheduler,
    ctx: Box<dyn Eval<AlgorithmQuery, EvalResult> + Send + Sync>,
}
//
//
impl SelectAlgorithm {
    ///
    /// Returns [SelectAlgorithm] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: AlgorithmConf,
        scheduler: Scheduler,
        ctx: impl Eval<AlgorithmQuery, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "SelectAlgorithm"),
            conf,
            scheduler,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl<K: Debug + Copy + bincode::Encode + Send + 'static> EvalEx<(Request<K>, Option<Link>), server::EvalResult<K>> for SelectAlgorithm {
    fn eval(&self, (req, link): (Request<K>, Option<Link>)) -> server::EvalResult<K> {
        let error = Error::new("SelectDevInfo", "eval");
        let query = extract!(&req.query, Query::Algorithm)
            .map_err(|_| error.err(format!("Query::DeviceInfo expected, but found {:?}", req.query_id)))?;
        //
        // Do something woth incomong query...
        // let param1 = query.param1;
        // let param2 = query.param2;
        //
        // Generate and return reply to the request
        match self.ctx.eval(query.clone()) {
            Ok(ctx) => {
                Ok(Some(req.reply(Reply::Algorithm(AlgorithmReply {}))))
            }
            Err(_) => todo!(),
        }
    }
    ///
    /// Halts hanbler
    fn exit(&self) {
        // Halt continuous operations here
    }
}
//
//
unsafe impl Send for SelectAlgorithm {}
