//! Entities implemented the Eval trate
pub mod stability;
pub mod strength;
pub mod criterion;
pub mod parameters;
pub mod icing_coeff;
pub mod icing_timber_bound;
pub mod icing_timber;
pub mod wetting;
pub mod unit_area;
pub mod draft_mark;
pub mod zg;
pub mod seakeeping;

pub use icing_coeff::ctx::IcingCoeffCtx;
pub use wetting::ctx::WettingCtx;
pub use unit_area::ctx::UnitAreaCtx;
pub use zg::ctx::ZgCtx;
pub use draft_mark::ctx::DraftMarkCtx;

