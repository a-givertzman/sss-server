use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, time::Duration};
use sal_sync::services::entity::{error::str_err::StrErr, name::Name, point::point_tx_id::PointTxId};
use crate::algorithm::entities::{area::HAreaStrength, strength::VerticalArea};

use super::{query::Query, reply::Reply};
///
/// Contains local side `send` & `recv` of `channel`
/// - provides simple direct to `send` & `recv`
/// - provides request operation
pub struct ModelLink {
    txid: usize,
    name: Name,
    send: Sender<Query>,
    recv: Option<Receiver<Reply>>,
    timeout: Duration,
    exit: Arc<AtomicBool>,
}
//
//
impl ModelLink {
    ///
    /// Default timeout to await `recv`` operation, 300 ms
    const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10);
    ///
    /// Returns [ModelLink] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(parent: impl Into<String>, send: Sender<Query>, recv: Receiver<Reply>) -> Self {
        let name = Name::new(parent, "ModelLink");
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            send, 
            recv: Some(recv),
            timeout: Self::DEFAULT_TIMEOUT,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns it's name
    pub fn name(&self) -> Name {
        self.name.clone()
    }
    // ///
    // /// Returns `local: [ModelLink] remote: [ModelLink]` new instance
    // pub fn split<>(parent: impl Into<String>) -> (Self, Self) {
    //     let name = Name::new(parent, "ModelLink");
    //     let (loc_send, rem_recv) = mpsc::channel();
    //     let (rem_send, loc_recv) = mpsc::channel();
    //     (
    //         Self { 
    //             txid: PointTxId::from_str(&name.join()),
    //             name: name.clone(),
    //             send: loc_send, recv: Some(loc_recv),
    //             timeout: Self::DEFAULT_TIMEOUT,
    //             exit: Arc::new(AtomicBool::new(false)),
    //         },
    //         Self { 
    //             txid: PointTxId::from_str(&name.join()),
    //             name,
    //             send: rem_send, recv: Some(rem_recv),
    //             timeout: Self::DEFAULT_TIMEOUT,
    //             exit: Arc::new(AtomicBool::new(false)),
    //         },
    //     )
    // }
    ///
    /// - Returns strength areas by ship frames
    pub async fn areas(&self) -> Result<(Vec<VerticalArea>, Vec<HAreaStrength>), StrErr> {
        let timeout = Duration::from_secs(1000);
        match self.send.send(Query::AreasStrength) {
            Ok(_) => {
                log::trace!("{}.areas | Sent request: {:#?}", self.name, Query::AreasStrength);
                tokio::task::block_in_place(move|| {
                    match &self.recv {
                        Some(recv) => match recv.recv_timeout(timeout) {
                            Ok(reply) => {
                                log::trace!("{}.req | Received reply: {:#?}", self.name, reply);
                                match reply {
                                    Reply::AreasStrength(items) => items,
                                    _ => panic!("{}.areas | Wrong reply: {:#?}", self.name, reply),
                                }
                            }
                            _ => Err(StrErr(format!("{}.req | Request timeout ({:?})", self.name, timeout))),
                        }
                        None => todo!(),
                    }
                })
            },
            Err(err) => Err(StrErr(format!("{}.req | Send request error: {:#?}", self.name, err))),
        }
    }
    ///
    /// Returns internal `exit` signal to be paired
    pub fn exit_pair(&self) -> Arc<AtomicBool> {
        self.exit.clone()
    }
    ///
    /// Sends "exit" signal to the `listen` task
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
//
//
unsafe impl Sync for ModelLink {}
//
//
impl Debug for ModelLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelLink")
        .field("txid", &self.txid)
        .field("name", &self.name)
        // .field("send", &self.send)
        // .field("recv", &self.recv)
        .field("timeout", &self.timeout)
        .finish()
    }
}
