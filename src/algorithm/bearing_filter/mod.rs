//! # Bearing Filtering Module
//! 
//! This module is responsible for filtering bearings based on predefined criteria.
//! It consists of two main components:
//! 
//! - `bearing_filter_ctx`: Stores the filtered bearings.
//! - `bearing_filter`: Implements the filtering logic.
//! 
//! ## Usage
//! 
//! The filtering process selects bearings that meet specific conditions related to static load capacity and outer diameter.
//! The results are stored in `BearingFilterCtx` and can be accessed within the algorithm's execution context.
//! 
//! For more details, refer to the design document:
//! [Filtering Bearings](design/docs/algorithm/part02/chapter_01_choose_hook.md)
//! 
//! ## Example
//! 
//! ```rust
//! use crate::algorithm::context::ctx_result::CtxResult;
//! use crate::kernel::eval::Eval;
//! use crate::algorithm::entities::bearing::Bearing;
//! use crate::algorithm::bearing_filter::BearingFilter;
//! 
//! let mut bearing_filter = BearingFilter::new(previous_step);
//! let result = bearing_filter.eval();
//! 
//! match result {
//!     CtxResult::Ok(ctx) => log::debug!("Filtered bearings: {:?}", ctx),
//!     CtxResult::Err(err) => log::debug!("Error: {}", err),
//!     CtxResult::None => log::debug!("No bearings matched the criteria."),
//! }
//! ```
//! 
//! ## Components
//! 
//! - `BearingFilterCtx`: Stores the list of filtered bearings.
//! - `BearingFilter`: Implements the evaluation logic for filtering bearings.
//! 
//! This module is a part of the crane design algorithm and is used to refine component selection.
pub mod bearing_filter_ctx;
pub mod bearing_filter;