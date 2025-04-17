use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle, time::Duration};
use bincode::{Decode, Encode};
use sal_core::error::Error;
use sal_sync::services::entity::{name::Name, point::point_tx_id::PointTxId};
use super::{link::Link, DEFAULT_TIMEOUT};
///
/// Combines multiple links
/// - Receives incomming events (requests) in the `listen` closure
/// - Provider `Sender` in the `listem` closure for sending reply
/// - Sends back the reply if returned from `listen` closure 
pub struct Hub {
    txid: usize,
    name: Name,
    links: Arc<papaya::HashMap<String, Link>>,
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
    pub fn new(parent: impl Into<String>) -> Self {
        let name = Name::new(parent, "Hub");
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            links: Arc::new(papaya::HashMap::new()),
            timeout: Duration::from_micros(100),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns new connected `Link`
    pub fn link(&self) -> Link {
        let (local, remote) = Link::split(&format!("{}:{}", self.name, self.links.len()));
        let key = remote.name().join();
        self.links.pin().insert(key, local);
        remote
    }
    ///
    /// Entry point
    pub fn listen<In: Decode<()> + Debug, Out: Encode + Debug>(&self, op: impl Fn(In) -> Option<Out> + Send + 'static) -> Result<JoinHandle<()>, Error> {
        let error = Error::new(&self.name, "listen");
        let dbg = self.name.join();
        let links = self.links.clone();
        let timeout = self.timeout;
        let exit = self.exit.clone();
        log::debug!("{}.listen | Starting...", dbg);
        let handle = std::thread::Builder::new().name(dbg.clone()).spawn(move|| {
            'main: loop {
                let links_pin = links.pin();
                let links_iter = links_pin.iter();
                for (id, link) in links_iter {
                    match link.recv_timeout(timeout) {
                        Ok(event) => {
                            match event {
                                Some(event) => {
                                    log::trace!("{}.listen | Link({id}) Received event: {:#?}", dbg, event);
                                    match (op)(event) {
                                        Some(reply) => {
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
                        Err(err) => log::warn!("{}.listen | Link({id}) Error: {:#?}", dbg, err),
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
            log::debug!("{}.listen | Exit", dbg);
        });
        let dbg = self.name.join();
        let error = Error::new(&self.name, "listen");
        log::debug!("{}.listen | Starting - Ok", dbg);
        handle.map_err(|err| error.pass(err.to_string()))
    }
    ///
    /// Sends "exit" signal to the service's task
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
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