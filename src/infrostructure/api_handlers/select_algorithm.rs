use std::{fmt::Debug, sync::{Arc, atomic::AtomicBool}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{sync::Handles, thread_pool::Scheduler};
use crate::{conf::AlgorithmConf, kernel::{Eval, EvalEx, sync::Link, types::eval_result::EvalResult}, server::{self, AlgorithmQuery, AlgorithmReply, Query, Reply, Request, extract}};

///
/// Evaluates entair ship calculations in the separate thread
pub struct SelectAlgorithm {
    conf: AlgorithmConf,
    scheduler: Scheduler,
    handles: Handles<()>,
    ctx: Box<dyn Eval<AlgorithmQuery, EvalResult> + Send + Sync>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
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
        let dbg = Dbg::new(parent, "SelectAlgorithm");
        Self {
            conf,
            scheduler,
            handles: Handles::new(&dbg),
            ctx: Box::new(ctx),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
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
        let h = self.scheduler.spawn(move || {
            match self.ctx.eval(query.clone()) {
                Ok(ctx) => {
                    let response = req.reply(Reply::Algorithm(AlgorithmReply {}));
                }
                Err(err) => {
                    let response = req.reply_err(error.pass_with("Calculations failed", err));
                }
            }
            Ok(())
        }).map_err(|err| error.pass_with("Can't spawn thread", err));
        match h {
            Ok(handle) => {
                self.handles.push(handle);
                Ok(None)
            }
            Err(err) => Err(err),
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
