use crate::algorithm::entities::Bounds;
use sal_core::error::Error;
use sal_sync::services::entity::{
 name::Name, point::point_tx_id::PointTxId,
};
use std::{
    fmt::Debug,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use super::{
    query::{BalanceSrcData, Query},
    reply::{BalanceResultData, Reply},
};
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
        }
    }
    ///
    /// Returns it's name
    pub fn name(&self) -> Name {
        self.name.clone()
    }
}
//
impl IModelLink for ModelLink {
    ///
    /// - Returns computed ship frames
    fn bounds(&self) -> Result<Bounds, Error> {
        let error = Error::new(&self.name, "bounds");
        let timeout = Duration::from_secs(300);
        match self.send.send(Query::Bounds) {
            Ok(_) => {
                log::debug!("{}.bounds | Sent request: {:#?}", self.name, Query::Bounds);
                match &self.recv {
                    Some(recv) => match recv.recv_timeout(timeout) {
                        Ok(reply) => {
                            log::debug!("{}.req | Received reply: {:#?}", self.name, reply);
                            match reply {
                                Reply::Bounds(items) => Ok(items),
                                _ => panic!("{}.bounds | Wrong reply: {:#?}", self.name, reply),
                            }
                        }
                        Err(_) => Err(error.err(format!("Request timeout ({:?})", timeout))),
                    },
                    None => todo!(),
                }
            }
            Err(err) => Err(error.pass_with("Send request error", err.to_string())),
        }
    }
    ///
    /// - Returns areas by ship frames
    fn bound_areas(&self) -> Result<(Vec<f64>, Vec<f64>), Error> {
        let error = Error::new(&self.name, "bound_areas");
        let timeout = Duration::from_secs(300);
        match self.send.send(Query::BoundAreas) {
            Ok(_) => {
                log::debug!(
                    "{}.bound_areas | Sent request: {:#?}",
                    self.name,
                    Query::BoundAreas
                );
                match &self.recv {
                    Some(recv) => match recv.recv_timeout(timeout) {
                        Ok(reply) => {
                            log::debug!("{}.req | Received reply: {:#?}", self.name, reply);
                            match reply {
                                Reply::BoundAreas(items) => items,
                                _ => panic!("{}.areas | Wrong reply: {:#?}", self.name, reply),
                            }
                        }
                        _ => Err(error.err(format!("Request timeout ({:?})", timeout))),
                    },
                    None => todo!(),
                }
            }
            Err(err) => Err(error.pass_with("Send request error: {:#?}", err.to_string())),
        }
    }
    /// - Returns areas by ship frames
    fn compute_balance(&self, data: BalanceSrcData) -> Result<BalanceResultData, Error> {
        let error = Error::new(&self.name, "bound_areas");
        let timeout = Duration::from_secs(300);
        match self.send.send(Query::ComputeBalance(data.clone())) {
            Ok(_) => {
                log::debug!(
                    "{}.compute_balance | Sent request: {:#?}",
                    self.name,
                    Query::ComputeBalance(data)
                );
                tokio::task::block_in_place(move || match &self.recv {
                    Some(recv) => match recv.recv_timeout(timeout) {
                        Ok(reply) => {
                            log::debug!("{}.req | Received reply: {:#?}", self.name, reply);
                            match reply {
                                Reply::ComputeBalance(reply) => reply,
                                _ => panic!(
                                    "{}.compute_balance | Wrong reply: {:#?}",
                                    self.name, reply
                                ),
                            }
                        }
                        _ => Err(StrErr(format!(
                            "{}.req | Request timeout ({:?})",
                            self.name, timeout
                        ))),
                    },
                    None => todo!(),
                })
            }
            Err(err) => Err(StrErr(format!(
                "{}.req | Send request error: {:#?}",
                self.name, err
            ))),
        }
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
//
pub trait IModelLink {
    fn bounds(&self) -> Result<Bounds, Error>;
    fn bound_areas(&self) -> Result<(Vec<f64>, Vec<f64>), Error>;
    fn compute_balance(&self, data: BalanceSrcData) -> Result<BalanceResultData, Error>;
}