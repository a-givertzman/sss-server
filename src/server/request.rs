use std::fmt::Debug;

use sal_core::error::Error;
use crate::server::{Content, Cot, Event, Query, Reply, Response};

///
/// The [Request] contains information about the Event and `Query`
/// - `T` - Defines content for later conversion `Message` payload bytes into specific type
#[derive(Debug, Clone)]
pub struct Request<QueryId> {
    /// Event id, used internal only to identify incoming request message
    pub event_id: u32,
    /// Name of the [Query], correspond with `query` variant
    pub query_id: QueryId,
    /// Cause of the transmission
    pub cot: Cot,
    /// Kind of the [Query] content
    pub content: Content,
    /// [Query] it self
    pub query: Query,
}
//
//
impl<QueryId: Debug + Copy> Request<QueryId> {
    ///
    /// Returns [Query] new instance
    /// - `id` - Query id, used internal only to identify incoming request message
    /// - `name` - Name of the [Query]
    /// - `bytes` - Payload bytes to be pased into concrete type
    pub fn new(event_id: u32, query_id: QueryId, cot: Cot, content: Content, query: Query) -> Self {
        Self {
            event_id,
            query_id,
            cot,
            content,
            query,
        }
    }
    ///
    /// Returns [Reply] to current [Request] with [Cot]::Con
    pub fn reply(&self, reply: Reply) -> Response<QueryId> {
        Response {
            event_id: self.event_id,
            query_id: self.query_id,
            cot: self.cot.reply_ok(),
            reply,
        }
    }
    ///
    /// Returns [Reply] to current [Request] with [Cot]::Inf
    pub fn reply_inf(&self, reply: Reply) -> Response<QueryId> {
        Response {
            event_id: self.event_id,
            query_id: self.query_id,
            cot: Cot::Inf,
            reply,
        }
    }
    ///
    /// Returns error [Reply] to current [Request]
    pub fn reply_err(&self, err: impl Into<String>) -> Response<QueryId> {
        Response {
            event_id: self.event_id,
            query_id: self.query_id,
            cot: self.cot.reply_err(),
            reply: Reply::error(err),
        }
    }
    ///
    /// Returns [Request] built from `Event`
    pub fn from_event(event: Event<QueryId>) -> Result<Request<QueryId>, Error> {
        let query = match event.content {
            Content::Any => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::Bool => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::Bytes =>  Query::from_bytes(&event.bytes),
            Content::Duration => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::Empty => Ok(Query::Empty),
            Content::F32 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::F64 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::I16 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::I32 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::I64 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::Json => Query::from_json(&event.bytes),
            Content::String => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::Timestamp => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::U16 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::U32 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
            Content::U64 => Err(Error::new("Request", "from_event").err(format!("Content {:?} - is not supported", event.content))),
        };
        match query {
            Ok(query) => Ok(Request::<QueryId> {
                event_id: event.id,
                query_id: event.query_id,
                cot: event.cot,
                content: event.content,
                query,
            }),
            Err(err) => Err(Error::new("Request", "from_event").pass_with(format!("Can't parse request {:?}", event.query_id), err)),
        }
    }
}
