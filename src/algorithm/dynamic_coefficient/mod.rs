//! # Module of calculating dynamic coefficient
//!
//! This module constist method of calculating steady state lifting speed
//! and their processing in the system.
//! [reference to dynamic coefficient documentation](design\docs\algorithm\part02\chapter_01_choose_hook.md)
//! It includes:
//! - calculating dynamic coefficient (`dynamic_coefficient`)
//!
//! ## Example of using
//! ```rust
//! use crate::algorithm::lifting_speed::lifting_speed::LiftingSpeed;
//! use crate::kernel::initial_data::initial_data::InitialData;
//! let path = "./src/tests/unit/kernel/storage/cache";
//! let dynamic_coefficient = let result = DynamicCoefficient::new(
//!                                             Arc::new(
//!                                                 RwLock::new(
//!                                                     Context::new(\
//!                                                         InitialCtx::new(
//!                                                             &mut Storage::new("./src/tests/unit/kernel/storage/cache/test_8")).unwrap()))))).eval();
//! println!("dynamic coefficient: {}", dynamic_coefficient);
//! ```
pub mod dynamic_coefficient;
pub mod dynamic_coefficient_ctx;
