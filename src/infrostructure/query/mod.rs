//! 
//! # Client - Server event interface description
//! 
//! ## ChooseUserHook Requiest
//! 
//! Ascs user for choosing hook from filtered hooks
//! 
//! - Cot::Req
//! 
//! ```json
//! {
//!     "req-name": "ChooseUserHook",
//!     "variant": [
//!                     Hook{
//!                     },
//!                     Hook{
//!                     },
//!                ]
//! }
//! ```
//! 
//! - Cot::ReqCon
//! 
//! ```json
//! {
//!     "req-name": "ChooseUserHook",
//!     "selection": "Selected Variant Id"
//! }
//! ```
//! 
pub mod query;
pub mod restart_eval;
