use std::fmt::Debug;
use sal_core::{dbg::Dbg, error::Error};
use serde::Serialize;
use crate::server::{Bytes, Content, Cot, Reply, Response};

///
/// The [Event] contains the name and data bytes to be sent over the socket
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub struct Event<QueryId> {
    /// Event id, used internal only to identify incoming request message
    pub id: u32,
    /// Name of the [Query], correspond with `query` variant
    pub query_id: QueryId,
    /// Cause of the transmission
    pub cot: Cot,
    /// Kind of the content in the `Data` field of  the socket `Message`
    pub content: Content,
    /// Payload data of the [Event], to be sent to the socket directly 
    pub bytes: Bytes,
}
//
//
impl<QueryId: Debug + Copy> Event<QueryId> {
    ///
    /// ## Returns [Event] new instance
    /// - `event_id` - Idendifier of the received message, take it from the `Request`
    /// - `query_id` - The name of the `Query`, take it from the `Request`
    /// - `cot` - Cause and diraction of the transmission, beter to use `Cot::reply()` method
    ///     - `Inf` - Information (Informational message, in general sent by backend to the client)
    ///     - `Act` - Activation (Command message, response is not required, optionally my be sent Cot::ActCon / Cot::ActErr)
    ///     - `ActCon` - Activation | Confirmatiom
    ///     - `ActErr` - Activation | Error
    ///     - `Req` - Request (Request message, Client expects response with Cot::ReqCon / Cot::ReqErr)
    ///     - `ReqCon` - Rquest | Confirmatiom reply 
    ///     - `ReqErr` - Rquest | Error reply
    /// - `bytes` - raw bytes to be sent over the socket
    pub fn new(id: u32, query_id: QueryId, cot: Cot, content: Content, bytes: Bytes) -> Self {
        Self {
            id,
            query_id,
            cot,
            content, 
            bytes,
        }
    }
    ///
    /// ## Json [Event] built from object
    /// 
    /// **Message [Content] type will selected automatically depend on the [Reply]**
    /// - [Reply::Empty] - will have [Content::Json]
    /// - [Reply::Bytes] - will have [Content::Bytes]
    /// - [Reply::Error] - will have [Content::Json]
    /// - [Reply::...] - will have [Content::Json]
    /// 
    /// - `dbg` - Parent dbg
    /// - `response` - Prepared [Response] contains `event_id`, `query_id`, `cot` and serializable `Reply` 
    pub fn from(dbg: &Dbg, response: Response<QueryId>) -> Self {
        match &response.reply {
            Reply::Empty => Self {
                id: response.event_id,
                query_id: response.query_id,
                cot: response.cot,
                content: Content::Empty,
                bytes: vec![],
            },
            Reply::Error(err) => Self::json(dbg, response.event_id, response.query_id, response.cot, err),
            Reply::Bytes(_) => Self {
                id: response.event_id,
                query_id: response.query_id,
                cot: response.cot,
                content: Content::Bytes,
                bytes: match response.reply {
                    Reply::Bytes(bytes) => bytes,
                    _ => panic!()
                },
            },
            _ => Self::json(dbg, response.event_id, response.query_id, response.cot, &response.reply),
        }
    }
    ///
    /// Returns [Event] with [Content::Json]
    fn json(dbg: &Dbg, event_id: u32, query_id: QueryId, cot: Cot, reply: impl Serialize) -> Self {
        match serde_json::to_vec(&reply) {
            Ok(bytes) => Self {
                id: event_id,
                query_id: query_id,
                cot: cot,
                content: Content::Json,
                bytes,
            },
            Err(err) => Self {
                id: event_id,
                query_id: query_id,
                cot: Self::error_cot(cot),
                content: Content::Json,
                bytes: serde_json::to_vec(
                    &Error::new(Dbg::new(dbg, "Event"), "json")
                        .pass_with(format!("Can't serialize response {:?}", query_id), err.to_string())
                        .to_string()
                ).unwrap_or(vec![]),
            },
        }
    }
    ///
    /// Returns error [Cot] depend on specified
    fn error_cot(cot: Cot) -> Cot {
        match cot {
            Cot::Inf => Cot::Inf,
            Cot::Act => Cot::ActErr,
            Cot::ActCon => Cot::ActErr,
            Cot::ActErr => Cot::ActErr,
            Cot::Req => Cot::ReqErr,
            Cot::ReqCon => Cot::ReqErr,
            Cot::ReqErr => Cot::ReqErr,
        }
    }
}