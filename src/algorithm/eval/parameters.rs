//!
//! Набор результатов расчетов для записи в БД
use bincode::{Decode, Encode};
use sal_core::error::Error;
use std::collections::HashMap;
use strum_macros::FromRepr;
///
/// Doc comment required
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr, Decode, Encode)]
pub enum ParameterID {
    CenterMassZFix = 1,
    Displacement = 2,
    DraughtMean = 3,
    DraughtBow = 4,
    DraughtStern = 5,
    TrimDeg = 6,
    Roll = 7,
    TonesPerCm = 8,
    MomentRollPerDeg = 9,
    MomentTrimPerCm = 10,
    CenterVolumeZ = 11,
    CenterMassZ = 12,
    MetacentricTransRad = 13,
    MetacentricTransRadZ = 14,
    MetacentricTransHeight = 15,
    MetacentricTransStore = 16,
    MetacentricTransBallast = 17,
    MetacentricTransHeightFix = 18,
    MetacentricLongRad = 19,
    MetacentricLongRadZ = 20,
    MetacentricLongHeight = 21,
    MetacentricLongStore = 22,
    MetacentricLongBallast = 23,
    MetacentricLongHeightFix = 24,
    MassBallast = 25,
    MassStores = 26,
    MassCargo = 27,
    MassDeadweight = 28,
    MassLightship = 29,
    MassIcing = 30,
    MassWetting = 31,
    CenterMassX = 32,
    WindPressure = 33,
    WindageArea = 34,
    WindageAreaLever = 35,
    StaticWindageHeelingLever = 36,
    DynamicWindageHeelingLever = 37,
    StaticWindageHeelingAngle = 38,
    DynamicWindageHeelingAngle = 39,
    HeelingAngleOfSecondPointOfIntersectionWith = 40,
    RollAmplitude = 41,
    RollPeriod = 42,
    AreaA = 43,
    AreaB = 44,
    OpenDeckEdgeImmersionAngle = 45,
    AngleOfDownFlooding = 46,
    SunsetAngle = 47,
    HeelingMomentDueToTheTransverseShiftOfGrain = 48,
    HeelingAngleWithMaximumDifference = 49,
    VesselSpeed = 50,
    TrimMeter = 51,
    CenterMassY = 52,
    CenterVolumeY = 53,
    CenterWaterlineAreaXFromStern = 54,
    CenterVolumeXFromStern = 55,
    CenterMassXFromStern = 56,
    MassBulkhead = 57,
    MassLightshipX = 58,
    MassBallastX = 59,
    MassStoresX = 60,
    MassCargoX = 61,
    MassIcingX = 62,
    MassWettingX = 63,
    MassBallastY = 64,
    MassStoresY = 65,
    MassCargoY = 66,
    MassBulkheadY = 67,
    MassIcingY = 68,
    MassWettingY = 69,
    MassBallastZ = 70,
    MassStoresZ = 71,
    MassCargoZ = 72,
    MassBulkheadZ = 73,
    MassIcingZ = 74,
    MassWettingZ = 75,
    MassBulkheadX = 76,
    MassLightshipY = 77,
    MassLightshipZ = 78,
    DraftSternSB = 79,
    DraftSternPS = 80,
    DraftSternAverage = 81,
    DraftSternIntermediateSB = 82,
    DraftSternIntermediatePS = 83,
    DraftSternIntermediateAverage = 84,
    DraftMidshipSB = 85,
    DraftMidshipPS = 86,
    DraftMidshipAverage = 87,
    DraftBowIntermediateSB = 88,
    DraftBowIntermediatePS = 89,
    DraftBowIntermediateAverage = 90,
    DraftBowSB = 91,
    DraftBowPS = 92,
    DraftBowAverageSB = 93,
    DraughtMid = 94,
    MetacentricTransSum = 95,
    CenterMassDeadweightZ = 96,
    CenterMassDeadweightY = 97,
    CenterMassDeadweightX = 98,
    HeelingLeverDueToTheTransverseShiftOfGrainWithZeroDifference = 99,
    HeelingLeverOfCurveWithMaximumDifference = 100,
    HeelingAngleDueToTheTransverseShiftOfGrain = 101,
    HeelingLeverOfDSOWithMaximumDifference = 102,
    HeelingLeverDueToTheTransverseShiftOfGrain = 103,
    MinimumOfFludingAngleSecondIntersectionAnd50Degrees = 104,
    HeelingLeverOfDSOCorrespondingToTheMinimumAngle = 105,
    HeelingLeverOfDSOCorrespondingToTheRollToTheWindwardSide = 106,
    RollToTheWindwardSide = 107,
    GrainArea = 108,
    WeightOfIceOnHorizontalSurfaces = 109,
    LongitudinalCenterOfWeightOfIceOnHorizontalSurfaces = 110,
    TransverseCenterOfWeightOfIceOnHorizontalSurfaces = 111,
    VerticalCenterOfWeightOfIceOnHorizontalSurfaces = 112,
    WeightOfIceOnVerticalSurfaces = 113,
    LongitudinalCenterOfWeightOfIceOnVerticalSurfaces = 114,
    TransverseCenterOfWeightOfIceOnVerticalSurfaces = 115,
    VerticalCenterOfWeightOfIceOnVerticalSurfaces = 116,
}
//
impl ParameterID {
    pub fn from(id: i32) -> Result<Self, Error> {
        let id = id as usize;
        ParameterID::from_repr(id).ok_or(Error::new("ParameterID", "from").err(format!("id:{id}")))
    }
}
/// Набор результатов расчетов для записи в БД
#[derive(Debug, Clone, Default)]
pub struct Parameters {
    data: HashMap<ParameterID, f64>,
}
//
impl IParameters for Parameters {
    /// Добавление нового параметра
    fn add(&mut self, id: ParameterID, value: f64) {
        self.data.insert(id, value);
    }
    /// Геттер, возвращает значение параметра или None если данных нет
    fn get(&self, id: ParameterID) -> Option<f64> {
        self.data.get(&id).copied()
    }
    /// Все данные в виде пар значений id/value
    fn take_data(self) -> Vec<(usize, f64)> {
        self.data
            .into_iter()
            .map(|(k, v)| (k as usize, v))
            .collect()
    }
}
//
#[doc(hidden)]
pub trait IParameters {
    /// Добавление нового параметра
    fn add(&mut self, id: ParameterID, value: f64);
    /// Геттер, возвращает значение параметра или None если данных нет
    fn get(&self, id: ParameterID) -> Option<f64>;
    /// Все данные в виде пар значений id/value
    fn take_data(self) -> Vec<(usize, f64)>;
}
// заглушка для тестирования
#[doc(hidden)]
pub struct FakeParameters;
#[doc(hidden)]
#[allow(dead_code)]
impl IParameters for FakeParameters {
    fn add(&mut self, _: ParameterID, _: f64) {}
    fn get(&self, _: ParameterID) -> Option<f64> {
        None
    }
    fn take_data(self) -> Vec<(usize, f64)> {
        Vec::new()
    }
}
