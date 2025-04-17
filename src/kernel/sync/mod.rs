mod hub;
mod link_sender;

use std::time::Duration;

///
/// Default timeout to await `recv`` operation, 300 ms
pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10);


pub use link_sender::LinkSend;
pub use hub::Hub;
pub mod link;
// pub mod recv_timeout;
pub mod switch;
