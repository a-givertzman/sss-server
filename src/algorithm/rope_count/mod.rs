//! # Rope Count Calculation Module
//!
//! This module is responsible for calculating the required number of ropes
//! based on the user's load capacity and total hook weight factors.
//!
//! It consists of two main components:
//!
//! - `rope_count_ctx`: Stores the calculated rope count value.
//! - `rope_count`: Implements the calculation logic.
//!
//! ## Usage
//!
//! The calculation process determines the necessary number of ropes
//! based on the load capacity, hook weight, and rope effort.
//! The results are stored in `RopeCountCtx` and can be accessed
//! within the algorithm's execution context.
//!
//! For more details, refer to the design document:
//! [Rope Count Calculation](design/docs/algorithm/part02/chapter_03_choose_hoisting_tackle.md)
//!
//! ## Example
//!
//! ```rust
//! use crate::algorithm::context::ctx_result::CtxResult;
//! use crate::kernel::eval::Eval;
//! use crate::algorithm::rope_count::RopeCount;
//!
//! let mut rope_count = RopeCount::new(previous_step);
//! let result = rope_count.eval();
//!
//! match result {
//!     CtxResult::Ok(ctx) => log::debug!("Calculated rope count: {:?}", ctx),
//!     CtxResult::Err(err) => log::debug!("Error: {}", err),
//!     CtxResult::None => log::debug!("No valid rope count could be determined."),
//! }
//! ```
//!
//! ## Components
//!
//! - `RopeCountCtx`: Stores the calculated number of ropes.
//! - `RopeCount`: Implements the evaluation logic for determining the required rope count.
//!
//! This module is a part of the crane design algorithm and is used to optimize rope selection.
pub mod rope_count_ctx;
pub mod rope_count;
