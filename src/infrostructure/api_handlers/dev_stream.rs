use std::{fmt::Debug, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::ServiceCycle, sync::Handles, thread_pool::Scheduler};
use crate::{kernel::{EvalEx, sync::Link}, server::{DevStreamConf, Device, EvalResult, Event, Reply, Request}};

///
/// Producess Device's events
pub struct DevStream {
    dbg: Dbg,
    conf: DevStreamConf,
    scheduler: Scheduler,
    is_active: Arc<AtomicBool>,
    handle: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
//
impl DevStream {
    ///
    /// Returns [SelectDevStream] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: DevStreamConf,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent.into(), "DevStream");
        Self {
            conf,
            scheduler,
            is_active: Arc::new(AtomicBool::new(false)),
            handle: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Wait for inner thread being finished
    #[allow(unused)]
    pub fn wait(&self) -> Result<(), Error> {
        self.handle.wait()
    }
}
//
//
impl<K: Debug + Copy + bincode::Encode + Send + 'static> EvalEx<(Request<K>, Option<Link>), EvalResult<K>> for DevStream {
    fn eval(&self, (req, link): (Request<K>, Option<Link>)) -> EvalResult<K> {
        let error = Error::new("DevStream", "eval");
        match self.is_active.load(Ordering::SeqCst) {
            true => Ok(None),
            false => match link {
                Some(link) => {
                    let dbg = self.dbg.clone();
                    let conf = self.conf.clone();
                    let exit = self.exit.clone();
                    log::info!("{dbg}.eval | Staring...");
                    let handle = self.scheduler.spawn(move || {
                        let mut devices: Vec<Device> = conf.devices.iter().map(|(_id, _conf)| {
                            Device {}
                        }).collect();
                        log::debug!("{dbg}.eval | Configured {} devices", devices.len());
                        let mut cycle = ServiceCycle::new(&dbg.to_string(), Duration::from_millis(10));
                        'main: loop {
                            cycle.start();
                            for dev in &mut devices {
                                *dev = Device::from(dev.clone());
                                //
                                // Prepare event & send Event
                                let event = Event::from(
                                    &dbg,
                                    req.reply(Reply::DeviceStream(dev.clone())),
                                );
                                if let Err(err) = link.send(event) {
                                    log::warn!("{dbg}.eval | Send error: {:?}", err);
                                }
                            }
                            if exit.load(Ordering::Acquire) {
                                break 'main;
                            }
                            cycle.wait();
                        }
                        log::info!("{dbg}.eval | Exit");
                        Ok(())
                    })
                        .map_err(|err| Error::new(&self.dbg, "eval").pass(err))?;
                    let dbg = self.dbg.clone();
                    self.handle.push(handle);
                    log::info!("{dbg}.eval | Staring - Ok");
                    Ok(None)
                }
                None => Err(error.err("Link is missing")),
            },
        }
    }
    ///
    /// Halts hanbler
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }
}
//
//
unsafe impl Send for DevStream {}
