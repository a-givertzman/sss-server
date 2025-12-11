//! Критерии проверки посадки судна

pub(crate) mod ctx;
pub(crate) mod eval;
mod load_line_eval;
//mod trim_eval;
mod bow_board_eval;
mod screw_eval;
mod reserve_buoyncy_eval;

pub use load_line_eval::eval::LoadLineEval;
pub use load_line_eval::ctx::LoadLineCtx;

//pub use trim_eval::trim_eval::TrimEval;
//pub use trim_eval::trim_ctx::TrimCtx;

pub use bow_board_eval::eval::BowBoardEval;
pub use bow_board_eval::ctx::BowBoardCtx;

pub use screw_eval::eval::ScrewEval;
pub use screw_eval::ctx::ScrewCtx;

pub use reserve_buoyncy_eval::eval::ReserveBuoyncyEval;
pub use reserve_buoyncy_eval::ctx::ReserveBuoyncyCtx;
