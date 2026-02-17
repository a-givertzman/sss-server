use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use bincode::{Decode, Encode};
use dashmap::DashMap;
use sal_core::error::Error;
use sal_sync::{services::entity::{Name, PointTxId}, thread_pool::{JoinHandle, Scheduler}};
use super::{link::Link, LinkSend};
///
/// Combines multiple links
/// - Receives incomming events (requests) in the `listen` closure
/// - Provider `Sender` in the `listem` closure for sending reply
/// - Sends back the reply if returned from `listen` closure 
pub struct Hub {
    txid: usize,
    name: Name,
    links: Arc<DashMap<String, Link>>,
    timeout: Duration,
    exit: Arc<AtomicBool>,
}
//
//
impl Hub {
    ///
    /// Returns [Hub] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(parent: impl Into<String>, exit: Option<Arc<AtomicBool>>) -> Self {
        let name = Name::new(parent, "Hub");
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            links: Arc::new(DashMap::new()),
            timeout: Duration::from_micros(100),
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// Returns new connected `Link`
    pub fn link(&self) -> Link {
        let (local, remote) = Link::split(&format!("{}:{}", self.name, self.links.len()));
        let key = remote.name().join();
        self.links.insert(key, local);
        remote
    }
    ///
    /// Listenning incomong events in the closure
    /// - Closure provides incoming event's
    /// - Send reoly
    ///     - Send it using [LinkSend] provided by closure
    ///     - Retur it from closure
    ///         - `Some<Event>` - will be sent as reply
    ///         - `None` - nothing will be sent
    pub fn listen<In: Decode<()> + Debug, Out: Encode + Debug>(
        &self,
        scheduler: Scheduler,
        op: impl Fn(In, LinkSend) -> Option<Out> + Send + 'static,
    ) -> Result<JoinHandle<()>, Error> {
        let error = Error::new(&self.name, "listen");
        let dbg = self.name.join();
        let links = self.links.clone();
        let timeout = self.timeout;
        let exit = self.exit.clone();
        log::trace!("{dbg}.listen | Start...");
        let handle = scheduler.spawn(move || {
            while !exit.load(Ordering::Acquire) {
                let mut closed_links = vec![];
                for entry in links.iter() {
                    let (id, link) = entry.pair();
                    match link.recv_timeout(timeout) {
                        Ok(event) => {
                            match event {
                                Some(event) => {
                                    log::trace!("{dbg}.listen | Link({id}) Received event: {:#?}", event);
                                    match (op)(event, link.sender()) {
                                        Some(reply) => {
                                            log::debug!("{dbg}.listen | Link({id}) Reply event: {:#?}", reply);
                                            if let Err(err) = link.send(reply) {
                                                let err = error.pass_with(format!("Link({id}) Send reply error"), err.to_string());
                                                log::error!("{}", err);
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                None => {}
                            }
                        }
                        Err(_) => closed_links.push(id.to_owned())
                    }
                    if exit.load(Ordering::Acquire) {
                        break;
                    }
                }
                for key in closed_links {
                    if let Some((_, removed)) = links.remove(&key) {
                        removed.exit();
                        log::trace!("{dbg}.listen | Link '{key}' - closed");
                    }
                }
            }
            // log::debug!("{dbg}.listen | Closing Links...");
            for link in links.iter() {
                link.exit();
            }
            // log::debug!("{dbg}.listen | Closing Links - Ok");
            log::debug!("{dbg}.listen | Exit");
            Ok(())
        });
        let dbg = self.name.join();
        let error = Error::new(&self.name, "listen");
        log::debug!("{dbg}.listen | Start - Ok");
        handle.map_err(|err| error.pass(err.to_string()))
    }
    ///
    /// Sends "exit" signal to the service's task
    pub fn exit(&self) {
        self.exit.store(true, Ordering::Release);
        for link in self.links.iter() {
            link.exit();
        }
    }
}
//
//
impl Debug for Hub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hub")
            .field("txid", &self.txid)
            .field("name", &self.name)
            // .field("send", &self.send)
            // .field("recv", &self.recv)
            // .field("subscribers", &self.subscribers)
            // .field("receivers", &self.receivers)
            .field("timeout", &self.timeout)
            .field("exit", &self.exit)
            .finish()
    }
}