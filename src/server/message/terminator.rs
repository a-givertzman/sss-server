use sal_core::error::Error;
use super::{Bytes, MessageParse};

///
/// Used to terminate the sequence of Message fields,
/// just returns passed bytes without changes
pub struct Terminator {}
//
//
impl Terminator {
    ///
    /// Returns [Terminator] new instance
    pub fn new() -> Self {
        Self {}
    }
}
impl<'a> MessageParse<(), (), Bytes> for Terminator {
    ///
    /// Resets passed `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<((), (), Bytes), Error> {
        Ok(((), (), bytes))
    }
}
