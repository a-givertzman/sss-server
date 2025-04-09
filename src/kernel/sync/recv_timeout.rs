use std::{sync::mpsc::RecvTimeoutError, time::Duration};
use sal_sync::services::entity::point::point::Point;
///
/// Provides `channel::recev` wirth specified timeout
pub trait RecvTimeout<T> {
    async fn recv_timeout(&mut self, duration: Duration) -> Result<T, RecvTimeoutError>;
}
impl RecvTimeout<Point> for tokio::sync::mpsc::Receiver<Point> {
    async fn recv_timeout(&mut self, duration: Duration) -> Result<Point, RecvTimeoutError> {
        match tokio::time::timeout(duration, self.recv()) {
            Ok(event) => match event {
                Some(event) => Ok(event),
                None => Err(RecvTimeoutError::Disconnected),
            }
            Err(_) => Err(RecvTimeoutError::Timeout),
        }
    }
}
