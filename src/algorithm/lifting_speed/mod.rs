//! # Module of calculating steady state lifting speed
//!
//! This module constist method of calculating steady state lifting speed
//! and their processing in the system.
//! [reference to steady-state lifting speed documentation](design\docs\algorithm\part02\chapter_01_choose_hook.md)
//! It includes:
//! - calculating steady state lifting speed (`steady_state_lifting_speed`)
//! - calculating half of nominal lifting speed of the mechanism (`payload_weight`)
//!
//! ## Example of using
//! ```rust
//! use crate::algorithm::lifting_speed::lifting_speed::LiftingSpeed;
//! use crate::kernel::initial_data::initial_data::InitialData;
//! let path = "....";
//! let lifting_speed = LiftingSpeed::new(Context::new(InitialCtx::new(&mut Storage::new(path)).expect("Error to create InitialCtx"))).eval();
//! println!("Steady state lifiting speed: {}", lifting_speed);
//! ```
pub mod lifting_speed;
pub mod lifting_speed_ctx;
