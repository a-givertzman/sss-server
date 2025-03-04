//! # Module where to store initial data and result's of all algorithm's
//!
//! This module constist of feild, that give all info about programm
//! and their processing in the system.
//!
//! ## Example of using
//! ```rust
//! use crate::{algorithm::context::{context::Context, ctx_result::CtxResult}
//! use crate::kernel::initial_ctx::initial_ctx::InitialCtx
//! let path = #"....";
//! let context = Contex::new(InitialCtx::new(Storage::new(path))).eval();
//! println!("Initial data: {}", context.initial);
//! ```
pub mod context_access;
pub mod context;
pub mod ctx_result;
///
/// TODO: To be moved to the better place
pub mod testing_ctx;
