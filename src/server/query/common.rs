//!
//! A list of common queries for the server
//! 
use bincode::Decode;
use serde::{Deserialize, Serialize};

///
/// Request for entair ship algorith evalueted
#[derive(Debug, Clone, Serialize, Deserialize, Decode)]
pub struct CalculusQuery {
    pub ship_id: usize,
    pub project_id: String,
}