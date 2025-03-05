use serde::{Serialize, Deserialize};
///
/// Client request | Restart of calculation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RestartEvalQuery {
    pub ship_id: usize,
}
///
/// Reply to [RestartEvalQuery]
#[derive(Debug, Serialize, Deserialize)]
pub struct RestartEvalReply {
    pub answer: u8
}
//
//
impl RestartEvalReply {
    ///
    /// New instance [InitialReply]
    pub fn new(choosen: u8) -> Self {
        Self {
            answer: choosen,
        }
    }
}
