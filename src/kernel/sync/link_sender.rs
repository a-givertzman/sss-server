use bincode::Encode;
use sal_core::error::Error;
use sal_sync::services::entity::Name;

use crate::kernel::types::channel::Sender;

///
/// Wrapper for `Sender<Vec<u8>>`,
/// provides sending generic `T`
/// internally encoded into `Vec<u8>`
pub struct LinkSend {
    name: Name,
    send: Sender<Vec<u8>>,
    config: bincode::config::Configuration,
}
//
//
impl LinkSend {
    ///
    /// Returns [LinkSend] new instance
    /// - send - `Sender<Vec<u8>>` - ower wich the data will be sent
    /// - conf - bincode::config::Configuration used for encoding the `T` into `Vec<u8>`
    pub fn new(parent: impl Into<String>, send: Sender<Vec<u8>>, config: bincode::config::Configuration) -> Self {
        Self {
            name: Name::new(parent, "LinkSend"),
            send,
            config,
        }
    }
    ///
    /// Sending event, generic `T` over the `Link`
    pub fn send(&self, event: impl Encode + std::fmt::Debug) -> Result<(), Error> {
        let error = Error::new(&self.name, "send");
        match bincode::encode_to_vec(event, self.config) {
            Ok(reply) => match self.send.send(reply) {
                Ok(_) => Ok(()),
                Err(err) => Err(error.pass(err.to_string())),
            }
            Err(err) => Err(error.pass_with("Encode error", err.to_string())),
        }
    }
}