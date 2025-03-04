//! # Rope Effort Calculation Module
//!
//! This module is responsible for calculating the required rope effort
//! based on the user's specified load capacity.
//!
//! It consists of two main components:
//!
//! - `rope_effort_ctx`: Stores the calculated rope effort value.
//! - `rope_effort`: Implements the calculation logic.
//!
//! ## Usage
//!
//! The calculation process determines the necessary rope effort based
//! on user loading capacity. The results are stored
//! in `RopeEffortCtx` and can be accessed within the algorithm's execution context.
//!
//! For more details, refer to the design document:
//! [Rope Effort Calculation](design/docs/algorithm/part02/chapter_03_choose_hoisting_tackle.md)
//!
//! ## Example
//!
//! ```rust
//! use crate::algorithm::context::ctx_result::CtxResult;
//! use crate::kernel::eval::Eval;
//! use crate::algorithm::rope_effort::RopeEffort;
//!
//! let mut rope_effort = RopeEffort::new(previous_step);
//! let result = rope_effort.eval();
//!
//! match result {
//!     CtxResult::Ok(ctx) => log::debug!("Calculated rope effort: {:?}", ctx),
//!     CtxResult::Err(err) => log::debug!("Error: {}", err),
//!     CtxResult::None => log::debug!("No valid load capacity provided."),
//! }
//! ```
//!
//! ## Components
//!
//! - `RopeEffortCtx`: Stores the calculated rope effort value.
//! - `RopeEffort`: Implements the evaluation logic for determining rope effort.
//!
//! This module is a part of the crane design algorithm and is used to optimize rope selection.
pub mod rope_effort_ctx;
pub mod rope_effort;
