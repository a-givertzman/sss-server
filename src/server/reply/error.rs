use serde::{Deserialize, Serialize};

///
/// Common error Reply
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorReply {
    pub code: ErrorCode,
    pub info: String,
}
//
//
impl ErrorReply {
    ///
    /// Returns [ErrorReply] new instance
    /// - `c` - `ErrorCode` contains proper information for end user
    /// - `e` - Technical details about the error
    #[allow(unused)]
    pub fn new(c: ErrorCode, e: impl Into<String>) -> Self {
        Self {
            code: c,
            info: e.into(),
        }
    }
    ///
    /// Returns [ErrorReply] new instance with `ErrorCode::Internal`, suitable for calculations errors
    /// - `e` - Technical details about the error
    pub fn internal(e: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InternalError,
            info: e.into(),
        }
    }
}
///
/// Common error codes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Client isn't athenticated for requested info
    Unauthorized,
    /// Requested info isn't exists, QueryId not matched to content of Data
    BadRequest,
    /// Server internal error
    InternalError,
    /// Requested info isn't implemented yet, but it's was  recognized as normal request
    NotImplemented,
}