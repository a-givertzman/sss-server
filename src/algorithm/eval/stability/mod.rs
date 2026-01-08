//! Entities implemented the Eval trate
pub mod icing;
pub mod static_mass;
pub mod balance;
pub mod metacentric_height;
pub mod lever_diagram;
pub mod wind;
pub mod windage;
pub mod roll_period;
pub mod roll_amplitude;

pub use icing::ctx::IcingStabCtx;
pub use static_mass::ctx::StaticMassCtx;
pub use balance::ctx::StabilityBalanceCtx;
pub use metacentric_height::ctx::MetacentricHeightCtx;
pub use lever_diagram::ctx::LeverDiagramCtx;
pub use windage::ctx::WindageCtx;
pub use wind::ctx::WindCtx;
pub use roll_period::ctx::RollingPeriodCtx;
pub use roll_amplitude::ctx::RollingAmplitudeCtx;

