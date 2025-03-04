use std::{fmt::Debug, sync::mpsc::{self, Receiver, Sender}, time::Duration};
use sal_sync::services::entity::{name::Name, point::{point::Point, point_tx_id::PointTxId}};
use serde::{de::DeserializeOwned, Serialize};
use crate::algorithm::context::ctx_result::CtxResult;
use super::str_err::str_err::StrErr;
///
/// Contains local side `send` & `recv` of `channel`
/// - provides simple direct to `send` & `recv`
/// - provides request operation
pub struct Link {
    txid: usize,
    name: Name,
    send: Sender<Point>,
    recv: Receiver<Point>,
    timeout: Duration,
}
//
//
impl Link {
    ///
    /// Default timeout to await `recv`` operation, 300 ms
    const DEFAULT_TIMEOUT: Duration = Duration::from_millis(300);
    ///
    /// Returns [Link] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(parent: impl Into<String>, send: Sender<Point>, recv: Receiver<Point>) -> Self {
        let name = Name::new(parent, "Link");
        Self {
            txid: PointTxId::from_str(&name.join()),
            name,
            send, 
            recv,
            timeout: Self::DEFAULT_TIMEOUT,
        }
    }
    ///
    /// Returns `local: [Link] remote: [Link]` new instance
    pub fn split(parent: impl Into<String>) -> (Self, Self) {
        let name = Name::new(parent, "Link");
        let (loc_send, rem_recv) = mpsc::channel();
        let (rem_send, loc_recv) = mpsc::channel();
        (
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name: name.clone(),
                send: loc_send, recv: loc_recv,
                timeout: Self::DEFAULT_TIMEOUT,
            },
            Self { 
                txid: PointTxId::from_str(&name.join()),
                name,
                send: rem_send, recv: rem_recv,
                timeout: Self::DEFAULT_TIMEOUT,
            },
        )
    }
    ///
    /// - Sends a request, 
    /// - Await reply,
    /// - Returns parsed reply
    pub fn req<T: DeserializeOwned + Debug>(&self, query: impl Serialize + Debug) -> Result<T, StrErr> {
        match serde_json::to_string(&query) {
            Ok(query) => {
                let query = Point::new(self.txid, &self.name.join(), query);
                match self.send.send(query) {
                    Ok(_) => {
                        match self.recv.recv_timeout(self.timeout) {
                            Ok(reply) => {
                                let reply = reply.as_string().value;
                                match serde_json::from_str::<T>(reply.as_str()) {
                                    Ok(reply) => {
                                        Ok(reply)
                                    }
                                    Err(err) => Err(StrErr(format!("{}.req | Deserialize error for {:?} in {}, \n\terror: {:#?}", self.name, std::any::type_name::<T>(), reply, err))),
                                }
                            }
                            _ => Err(StrErr(format!("{}.req | Request timeout ({:?})", self.name, self.timeout))),

                        }
                    },
                    Err(err) => Err(StrErr(format!("{}.req | Send request error: {:#?}", self.name, err))),
                }
            }
            Err(err) => Err(StrErr(format!("{}.req | Serialize query error: {:#?}, \n\tquery: {:#?}", self.name, err, query))),
        }
    }
    ///
    /// Receiving incomong events
    /// - Returns Ok<T> if channel has query
    /// - Returns None if channel is empty for now
    /// - Returns Err if channel is closed
    pub fn recv_query<T: DeserializeOwned + Debug>(&self) -> CtxResult<T, StrErr> {
        match self.recv.recv_timeout(self.timeout) {
            Ok(quyru) => {
                let quyru = quyru.as_string().value;
                match serde_json::from_str::<T>(quyru.as_str()) {
                    Ok(query) => {
                        return CtxResult::Ok(query)
                    }
                    Err(err) => CtxResult::Err(
                        StrErr(
                            format!("{}.req | Deserialize error for {:?} in {}, \n\terror: {:#?}",
                            self.name, std::any::type_name::<T>(), quyru, err),
                        ),
                    ),
                }
            }
            Err(err) => {
                match err {
                    mpsc::RecvTimeoutError::Timeout => CtxResult::None,
                    mpsc::RecvTimeoutError::Disconnected => CtxResult::Err(
                        StrErr(format!("{}.req | Recv error: {:#?}", self.name, err)),
                    ),
                }
            }
        }
    }
    ///
    /// Sending event
    pub fn send_reply(&self, reply: impl Serialize + Debug) -> Result<(), StrErr> {
        match serde_json::to_string(&reply) {
            Ok(reply) => {
                let reply = Point::new(self.txid, &self.name.join(), reply);
                match self.send.send(reply) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(StrErr(format!("{}.reply | Send request error: {:#?}", self.name, err))),
                }
            }
            Err(err) => Err(StrErr(format!("{}.reply | Serialize reply error: {:#?}, \n\tquery: {:#?}", self.name, err, reply))),
        }
    }
}
//
//
unsafe impl Sync for Link {}
//
//
impl Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Link")
        .field("txid", &self.txid)
        .field("name", &self.name)
        // .field("send", &self.send)
        // .field("recv", &self.recv)
        .field("timeout", &self.timeout)
        .finish()
    }
}