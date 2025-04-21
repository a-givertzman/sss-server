//! Критерии проверки остойчивости судна
use strum_macros::FromRepr;

pub mod criterion_stability_ctx;
pub mod criterion_stability_eval;

pub mod wheather_eval;
pub mod static_angle_eval;
pub mod dso_area_eval;
pub mod dso_max_eval;

pub use wheather_eval::wheather_eval::WheatherEval;
pub use wheather_eval::wheather_ctx::WheatherCtx;

pub use static_angle_eval::static_angle_eval::StaticAngleEval;
pub use static_angle_eval::static_angle_ctx::StaticAngleCtx;

pub use dso_area_eval::dso_area_eval::DSOAreaEval;
pub use dso_area_eval::dso_area_ctx::DSOAreaCtx;

pub use dso_max_eval::dso_max_eval::DSOMaxEval;
pub use dso_max_eval::dso_max_ctx::DSOMaxCtx;

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
    pub fn from(id: i32) -> Result<Self, crate::Error> {
        let id = id as usize;
        CriterionID::from_repr(id).ok_or(Error::FromString(format!(
            "CriterionID from_usize error: {}",
            id
        )))
    }
}
