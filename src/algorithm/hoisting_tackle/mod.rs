//! # Hoisting Tackle Selection Module
//!
//! This module is responsible for selecting and managing the hoisting tackle
//! during the crane configuration process.
//!
//! ## Structure
//!
//! It consists of two main components:
//!
//! - `hoisting_tackle_ctx`: Stores the selected hoisting tackle configuration.
//! - `hoisting_tackle`: Implements the logic for evaluating and adjusting the hoisting tackle.
//!
//! ## Usage
//!
//! The hoisting tackle selection process determines the appropriate configuration
//! based on user input or predefined parameters. If necessary, the system can
//! request user confirmation or modification.
//!
//! For more details, refer to the design document:
//! [Hoisting Tackle Selection](design/docs/algorithm/part02/chapter_03_choose_hoisting_tackle.md)
//!
//! ## Example
//!
//! ```rust
//! use crate::algorithm::context::ctx_result::CtxResult;
//! use crate::kernel::eval::Eval;
//! use crate::algorithm::hoisting_tackle::HoistingTackle;
//!
//! let mut hoisting_tackle = HoistingTackle::new(link, request, previous_step);
//! let result = hoisting_tackle.eval();
//!
//! match result {
//!     CtxResult::Ok(ctx) => log::debug!("Selected hoisting tackle: {:?}", ctx),
//!     CtxResult::Err(err) => log::debug!("Error: {}", err),
//!     CtxResult::None => log::debug!("No valid selection provided."),
//! }
//! ```
//!
//! ## Components
//!
//! - `HoistingTackleCtx`: Stores the selected hoisting tackle value.
//! - `HoistingTackle`: Implements the selection and user interaction logic.
//!
//! This module is an essential part of the crane design algorithm, ensuring
//! that the correct hoisting tackle is chosen for safe and efficient operation.

pub mod hoisting_tackle_ctx;
pub mod hoisting_tackle;
