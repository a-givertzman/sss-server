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
//! - Cot::ReqErr
//! 
//! ```json
//! {
//!     "req-name": "ChooseUserHook",
//!     "error": "Error Id"
//! }
//! ```
//! 
//! //! ## ChooseUserBearing Requiest
//! 
//! Ascs user for choosing bearing from filtered bearings
//! 
//! - Cot::Req
//! 
//! ```json
//! {
//!     "req-name": "ChooseUserBearing",
//!     "variant": [
//!                     Bearing{
//!                     },
//!                     Bearing{
//!                     },
//!                ]
//! }
//! ```
//! 
//! - Cot::ReqCon
//! 
//! ```json
//! {
//!     "req-name": "ChooseUserBearing",
//!     "selection": "Selected Variant Id"
//! }
//! ```
//! 
//! - Cot::ReqErr
//! 
//! ```json
//! {
//!     "req-name": "ChooseUserBearing",
//!     "error": "Error Id"
//! }
//! ```
//! 
//! ## ChooseHoistingRope Requiest
//! 
//! Ascs user for choosing hoisting rope from filtered hoisting ropes
//! 
//! - Cot::Req
//! 
//! ```json
//! {
//!     "req-name": "ChooseHoistingRope",
//!     "variant": [
//!                     HoistingRope{
//!                     },
//!                     HoistingRope{
//!                     },
//!                ]
//! }
//! ```
//! 
//! - Cot::ReqCon
//! 
//! ```json
//! {
//!     "req-name": "ChooseHoistingRope",
//!     "selection": "Selected Variant Id"
//! }
//! ```
//! 
//! - Cot::ReqErr
//! 
//! ```json
//! {
//!     "req-name": "ChooseHoistingRope",
//!     "error": "Error Id"
//! }
//! ```
//! ## ChangeHoistingTackle Requiest
//! 
//! Ascs user for changing hoisting tackle if it needed
//! 
//! - Cot::Req
//! 
//! ```json
//! {
//!     "req-name": "ChangeHoistingTackle",
//!     "variant": [
//!                     0,
//!                     1,
//!                     2,
//!                ]
//! }
//! ```
//! 
//! - Cot::ReqCon
//! 
//! ```json
//! {
//!     "req-name": "ChangeHoistingTackle",
//!     "selection": "Selected Variant Id"
//! }
//! ```
//! 
//! - Cot::ReqErr
//! 
//! ```json
//! {
//!     "req-name": "ChangeHoistingTackle",
//!     "error": "Error Id"
//! }
//! ```
pub mod change_hoisting_tackle;
pub mod choose_hoisting_rope;
pub mod choose_user_bearing;
pub mod choose_user_hook;
pub mod query;
