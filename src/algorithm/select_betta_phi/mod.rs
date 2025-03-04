//! # Module of choice betta and phi coefficients
//!
//! This module constist method of choosing betta and phi coefficients
//! and their processing in the system.
//! [reference to choice documentation](design\docs\algorithm\part02\chapter_01_choose_hook.md)
//! It includes:
//! - choosing method of betta and phi coefficients (`bet_phi`)
//!
//! ## Example of using
//! ```rust
//! use crate::algorithm::select_betta_phi::select_betta_phi::SelectBettaPhi
//! use crate::kernel::initial_data::initial_data::InitialData;
//! let path = "..."
//! let bet_phi = SelectBettaPhi::new(Arc::new(
//!                                 RwLock::new(
//!                                     Context::new(
//!                                     InitialCtx::new(&mut Storage::new(path,)).unwrap())).clone()).eval()
//! println!("Betta and Phi coefficients: {}", total);
//! ```
pub mod select_betta_phi;
pub mod select_betta_phi_ctx;
