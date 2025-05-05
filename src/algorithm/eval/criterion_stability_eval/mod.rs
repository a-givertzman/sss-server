//! Критерии проверки остойчивости судна

use strum_macros::FromRepr;
use sal_core::error::Error;
use std::fmt::Debug;

pub mod criterion_stability_ctx;
pub mod criterion_stability_eval;
pub mod parameters;
pub mod wheather_eval;
pub mod static_angle_eval;
pub mod dso_area_eval;
pub mod dso_max_eval;
pub mod dso_timber_max_eval;
pub mod dso_icing_max_eval;
pub mod dso_angle_max_eval;
pub mod min_metacentric_height_eval;
pub mod acceleration_eval;
pub mod circulation_eval;
pub mod grain_eval;
pub mod load_line_eval;
//pub mod trim_eval;
pub mod bow_board_eval;
pub mod screw_eval;
//pub mod reserve_buoyncy_eval;

pub use wheather_eval::wheather_eval::WheatherEval;
pub use wheather_eval::wheather_ctx::WheatherCtx;

pub use static_angle_eval::static_angle_eval::StaticAngleEval;
pub use static_angle_eval::static_angle_ctx::StaticAngleCtx;

pub use dso_area_eval::dso_area_eval::DSOAreaEval;
pub use dso_area_eval::dso_area_ctx::DSOAreaCtx;

pub use dso_max_eval::dso_max_eval::DSOMaxEval;
pub use dso_max_eval::dso_max_ctx::DSOMaxCtx;

pub use dso_timber_max_eval::dso_timber_max_eval::DSOTimberMaxEval;
pub use dso_timber_max_eval::dso_timber_max_ctx::DSOTimberMaxCtx;

pub use dso_icing_max_eval::dso_icing_max_eval::DSOIcingMaxEval;
pub use dso_icing_max_eval::dso_icing_max_ctx::DSOIcingMaxCtx;

pub use dso_angle_max_eval::dso_angle_max_eval::DSOAngleMaxEval;
pub use dso_angle_max_eval::dso_angle_max_ctx::DSOAngleMaxCtx;

pub use min_metacentric_height_eval::min_metacentric_height_eval::MinMetacentricHeightEval;
pub use min_metacentric_height_eval::min_metacentric_height_ctx::MinMetacentricHeightCtx;

pub use acceleration_eval::acceleration_eval::AccelerationEval;
pub use acceleration_eval::acceleration_ctx::AccelerationCtx;

pub use circulation_eval::circulation_eval::CirculationEval;
pub use circulation_eval::circulation_ctx::CirculationCtx;

pub use grain_eval::grain_eval::GrainEval;
pub use grain_eval::grain_ctx::GrainCtx;

pub use load_line_eval::load_line_eval::LoadLineEval;
pub use load_line_eval::load_line_ctx::LoadLineCtx;

//pub use trim_eval::trim_eval::TrimEval;
//pub use trim_eval::trim_ctx::TrimCtx;

pub use bow_board_eval::bow_board_eval::BowBoardEval;
pub use bow_board_eval::bow_board_ctx::BowBoardCtx;

pub use screw_eval::screw_eval::ScrewEval;
pub use screw_eval::screw_ctx::ScrewCtx;

//pub use reserve_buoyncy_eval::reserve_buoyncy_eval::ReserveBuoyncyEval;
//pub use reserve_buoyncy_eval::reserve_buoyncy_ctx::ReserveBuoyncyCtx;


#[derive(Hash, Eq, PartialEq, FromRepr)]
pub enum CriterionID {
    Wheather = 1,
    WindStaticHeel = 2,
    AreaLC0_30 = 3,
    AreaLc0Thetalmax = 4,
    AreaLC0_40 = 5,
    AreaLC30_40 = 6,
    MaximumLC = 7,
    MaximumLcTimber = 8,
    MaximumLcIcing = 9,
    HeelMaximumLC = 10,
    HeelFirstMaximumLC = 11,
    MinMetacentricHight = 12,
    Acceleration = 13,
    HeelTurning = 14,
    HeelGrainDisplacement = 15,
    AreaLcGrainDisplacement = 16,
    MinMetacentricHeightSubdivIndex = 17,
    LlDraftSSB = 101,
    LlDraftSPS = 102,
    LlDraftWSB = 103,
    LlDraftWPS = 104,
    LlDraftWNASB = 105,
    LlDraftWNAPS = 106,
    LlDraftTSB = 107,
    LlDraftTPS = 108,
    LlDraftFSB = 109,
    LlDraftFPS = 110,
    LlDraftTFSB = 111,
    LlDraftTFPS = 112,
    LlDraftLSSB = 113,
    LlDraftLSPS = 114,
    LlDraftLWSB = 115,
    LlDraftLWPS = 116,
    LlDraftLWNASB = 117,
    LlDraftLWNAPS = 118,
    LlDraftLTSB = 119,
    LlDraftLTPS = 120,
    LlDraftLFSB = 121,
    LlDraftLFPS = 122,
    LlDraftLTFSB = 123,
    LlDraftLTFPS = 124,
    LlDraftSI1Reserve = 125,
    LlDraftSI16Reserve = 140,
    MaximumForwardTrim = 141,
    MaximumAftTrim = 142,
    DepthAtForwardPerpendicularSB = 143,
    DepthAtForwardPerpendicularPS = 144,
    ScrewImmersionCL = 145,
    ScrewImmersionSB = 146,
    ScrewImmersionPS = 147,
    ScrewImmersionReserve = 148,
    //  ScrewImmersionRreserve = 149,
    ReserveBuoyncyInBow = 150,
}
//
impl CriterionID {
    pub fn from(id: i32) -> Result<Self, Error> {
        let id = id as usize;
        CriterionID::from_repr(id)
            .ok_or(Error::new("CriterionID", "from").err(format!("id:{id}")))
    }
}
/// Результат проверки критерия
#[derive(Clone)]
pub struct CriterionData {
    /// id критерия
    pub criterion_id: usize,
    /// Результат расчета
    pub result: f64,
    /// Пороговое значение критерия
    pub target: f64,
    /// Текст ошибки
    pub error_message: Option<String>,
}
//
impl CriterionData {
    /// Конструктор при наличии результата
    pub fn new_result(criterion_id: CriterionID, result: f64, target: f64) -> Self {
        Self {
            criterion_id: criterion_id as usize,
            result,
            target,
            error_message: None,
        }
    }
    /// Конструктор при ошибке расчета
    pub fn new_error(criterion_id: CriterionID, error_message: String) -> Self {
        Self {
            criterion_id: criterion_id as usize,
            result: 0.,
            target: 0.,
            error_message: Some(error_message),
        }
    }
}
//
impl Debug for CriterionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CriterionData")
            .field("criterion_id", &self.criterion_id)
            .field("result", &self.result)
            .field("target", &self.target)
            .field("error_message", &self.error_message)
            .finish()
    }
}
