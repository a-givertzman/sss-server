//! # User Selection Module
//!
//! This module handles user input for selecting bearings and hooks during the crane design process.
//! It consists of four main components:
//!
//! - `user_bearing_ctx`: Stores the selected user bearing.
//! - `user_bearing`: Handles user interaction for bearing selection.
//! - `user_hook_ctx`: Stores the selected user hook.
//! - `user_hook`: Handles user interaction for hook selection.
//!
//! ## Usage
//!
//! This module provides a way for the user to manually choose a bearing and a hook based on available options.
//! The selection is stored in `UserBearingCtx` and `UserHookCtx`, which can be accessed within the algorithm's execution context.
//!
//! For more details, refer to the design document:
//! [User Selection](design/docs/algorithm/part02/chapter_01_choose_hook.md)
//!
//! ## Example
//!
//! ```rust
//! use crate::algorithm::context::ctx_result::CtxResult;
//! use crate::kernel::eval::Eval;
//! use crate::algorithm::user_bearing::UserBearing;
//! use crate::algorithm::user_hook::UserHook;
//! use crate::kernel::request::Request;
//! use crate::algorithm::entities::bearing::Bearing;
//! use crate::algorithm::entities::hook::Hook;
//!
//! let bearing_request = Request::<Bearing>::new();
//! let hook_request = Request::<Hook>::new();
//!
//! let mut user_bearing = UserBearing::new(bearing_request, previous_step);
//! let mut user_hook = UserHook::new(hook_request, previous_step);
//!
//! let bearing_result = user_bearing.eval();
//! let hook_result = user_hook.eval();
//!
//! match (bearing_result, hook_result) {
//!     (CtxResult::Ok(ctx_b), CtxResult::Ok(ctx_h)) => log::debug!("User selections: {:?}, {:?}", ctx_b, ctx_h),
//!     _ => log::debug!("Error in user selection."),
//! }
//! ```
//!
//! ## Components
//!
//! - `UserBearingCtx`: Stores the selected bearing chosen by the user.
//! - `UserBearing`: Implements the logic for requesting a bearing from the user.
//! - `UserHookCtx`: Stores the selected hook chosen by the user.
//! - `UserHook`: Implements the logic for requesting a hook from the user.
//!
//! This module ensures that the user can participate in the selection process while maintaining the structured evaluation pipeline.
pub mod user_bearing_ctx;
pub mod user_bearing;
pub mod user_hook_ctx;
pub mod user_hook;
