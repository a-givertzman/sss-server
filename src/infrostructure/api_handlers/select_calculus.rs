use std::{fmt::Debug, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{sync::Handles, thread_pool::Scheduler};
use crate::{
    conf::CalculusConf,
    kernel::{EvalEx, sync::Link, types::eval_result::EvalResult},
    server::{self, CalculusQuery, CalculusReply, CalculusStatus, Event, Query, Reply, Request, extract},
};

///
/// Evaluates entair ship calculations in the separate thread
pub struct SelectCalculus {
    conf: CalculusConf,
    scheduler: Scheduler,
    handles: Handles<()>,
    ctx: Arc<Box<dyn EvalEx<CalculusQuery, EvalResult> + Send + Sync>>,
    in_progress: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl SelectCalculus {
    ///
    /// Returns [SelectAlgorithm] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: CalculusConf,
        scheduler: Scheduler,
        ctx: impl EvalEx<CalculusQuery, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "SelectAlgorithm");
        Self {
            conf,
            scheduler,
            handles: Handles::new(&dbg),
            ctx: Arc::new(Box::new(ctx)),
            in_progress: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
}
//
//
impl<K: Debug + Copy + bincode::Encode + Send + 'static> EvalEx<(Request<K>, Option<Link>), server::EvalResult<K>> for SelectCalculus {
    fn eval(&self, (req, link): (Request<K>, Option<Link>)) -> server::EvalResult<K> {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "eval");
        let link = link.ok_or(error.err("Can't get Link"))?;
        let in_progress = self.in_progress.clone();
        let exit = self.exit.clone();
        let query = extract!(&req.query, Query::Calculus).cloned()
            .map_err(|_| error.err(format!("Query::DeviceInfo expected, but found {:?}", req.query_id)))?;
        //
        // Do something woth incomong query...
        // let param1 = query.param1;
        // let param2 = query.param2;
        //
        let ctx = self.ctx.clone();
        let error1 = error.clone();
        if in_progress.load(Ordering::Acquire) {
            ctx.exit();
            let response = req.reply(Reply::Calculus(CalculusReply { status: CalculusStatus::Canceled }));
            if let Err(err) = link.send(Event::from(&dbg, response)) {
                log::warn!("{dbg}.eval | Can't send reply: {:?}", err);
            }
        }
        let response = req.reply(Reply::Calculus(CalculusReply { status: CalculusStatus::Ongoing }));
        if let Err(err) = link.send(Event::from(&dbg, response)) {
            log::warn!("{dbg}.eval | Can't send reply: {:?}", err);
        }
        let h = self.scheduler.spawn(move || {
            in_progress.store(true, Ordering::Release);
            //
            // Generate and return reply to the request
            let response = match ctx.eval(query.clone()) {
                Ok(ctx) => {
                    in_progress.store(false, Ordering::Release);
                    //
                    // Do required operations with the Context
                    //
                    // Then send reply
                    req.reply(Reply::Calculus(CalculusReply { status: CalculusStatus::Done }))
                }
                Err(err) => {
                    req.reply_err(error1.pass_with("Calculations failed", err))
                }
            };
            if let Err(err) = link.send(Event::from(&dbg, response)) {
                log::warn!("{dbg}.eval | Can't send reply: {:?}", err);
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
        self.exit.store(true, Ordering::Release);
    }
}
//
//
unsafe impl Send for SelectCalculus {}
