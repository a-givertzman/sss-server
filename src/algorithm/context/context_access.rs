use super::context::Context;
use crate::algorithm::{
    context::testing_ctx::TestingCtx,
    eval::{
        
        icing_timber::ctx::IcingTimberCtx,
        icing_timber_bound::ctx::IcingTimberBoundCtx, 
        parameters::*,
        strength::*,  
        stability::*,
        criterion::*, 
        *,
    },
    initial::initial_ctx::InitialCtx,
};
use sal_core::error::Error;
///
/// Provides restricted write access to the [Context] members
pub trait ContextWrite<T> {
    fn write(self, value: T) -> Result<Context, Error>;
}
///
/// Provides simple read access to the [Context] members
pub trait ContextReadRef<T> {
    fn read_ref(&self) -> &T;
}
///
/// Provides simple read access to the [Context] members
pub trait ContextRead<T> {
    fn read(&self) -> T;
}
///
/// Provides restricted write access to the [Context].[Parameters] members
pub trait ContextParamsWrite {
    fn write_params(&mut self, key: ParameterID, value: f64);
}
///
/// Provides simple read access to the [Context].[Parameters] members
pub trait ContextParamsRead {
    fn read_params(&self, key: ParameterID) -> f64;
}

//
impl ContextReadRef<Parameters> for Context {
    fn read_ref(&self) -> &Parameters {
        &self.parameters
    }
}
impl ContextParamsWrite for Context {
    fn write_params(&mut self, id: ParameterID, value: f64) {
        self.parameters.add(id, value);
    }
}
impl ContextParamsRead for Context {
    fn read_params(&self, id: ParameterID) -> f64 {
        let params: &Parameters = self.read_ref();
        params
            .get(id)
            .expect(&format!("Context.read | Id '{:?}' - is not found", id))
    }
}
//
impl ContextWrite<InitialCtx> for Context {
    fn write(mut self, value: InitialCtx) -> Result<Self, Error> {
        self.initial = value;
        Result::Ok(self)
    }
}
impl ContextReadRef<InitialCtx> for Context {
    fn read_ref(&self) -> &InitialCtx {
        &self.initial
    }
}
//
impl ContextWrite<TestingCtx> for Context {
    fn write(mut self, value: TestingCtx) -> Result<Self, Error> {
        self.testing = Some(value);
        Result::Ok(self)
    }
}
impl ContextReadRef<Option<TestingCtx>> for Context {
    fn read_ref(&self) -> &Option<TestingCtx> {
        &self.testing
    }
}
//
impl ContextWrite<IcingCoeffCtx> for Context {
    fn write(mut self, value: IcingCoeffCtx) -> Result<Self, Error> {
        self.icing_coeff = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<IcingCoeffCtx> for Context {
    fn read(&self) -> IcingCoeffCtx {
        self.icing_coeff.clone().unwrap()
    }
}
//
impl ContextWrite<IcingTimberBoundCtx> for Context {
    fn write(mut self, value: IcingTimberBoundCtx) -> Result<Self, Error> {
        self.icing_timber_bound = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<IcingTimberBoundCtx> for Context {
    fn read(&self) -> IcingTimberBoundCtx {
        self.icing_timber_bound.clone().unwrap()
    }
}
//
impl ContextWrite<IcingTimberCtx> for Context {
    fn write(mut self, value: IcingTimberCtx) -> Result<Self, Error> {
        self.icing_timber = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<IcingTimberCtx> for Context {
    fn read(&self) -> IcingTimberCtx {
        self.icing_timber.clone().unwrap()
    }
}
//
impl ContextWrite<UnitAreaCtx> for Context {
    fn write(mut self, value: UnitAreaCtx) -> Result<Self, Error> {
        self.unit_area = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<UnitAreaCtx> for Context {
    fn read(&self) -> UnitAreaCtx {
        self.unit_area.clone().unwrap()
    }
}
//
impl ContextWrite<WettingCtx> for Context {
    fn write(mut self, value: WettingCtx) -> Result<Self, Error> {
        self.wetting = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<WettingCtx> for Context {
    fn read(&self) -> WettingCtx {
        self.wetting.clone().unwrap()
    }
}
//
impl ContextWrite<AreaStrCtx> for Context {
    fn write(mut self, value: AreaStrCtx) -> Result<Self, Error> {
        self.area_str = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<AreaStrCtx> for Context {
    fn read(&self) -> AreaStrCtx {
        self.area_str.clone().unwrap()
    }
}
//
impl ContextWrite<IcingStrCtx> for Context {
    fn write(mut self, value: IcingStrCtx) -> Result<Self, Error> {
        self.icing_str = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<IcingStrCtx> for Context {
    fn read(&self) -> IcingStrCtx {
        self.icing_str.clone().unwrap()
    }
}
//
impl ContextWrite<StaticMassStrCtx> for Context {
    fn write(mut self, value: StaticMassStrCtx) -> Result<Self, Error> {
        self.static_mass_str = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<StaticMassStrCtx> for Context {
    fn read(&self) -> StaticMassStrCtx {
        self.static_mass_str.clone().unwrap()
    }
}
//
impl ContextWrite<StrengthBalanceCtx> for Context {
    fn write(mut self, value: StrengthBalanceCtx) -> Result<Self, Error> {
        self.strength_balance = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<StrengthBalanceCtx> for Context {
    fn read(&self) -> StrengthBalanceCtx {
        self.strength_balance.clone().unwrap()
    }
}
//
impl ContextWrite<DynamicMassCtx> for Context {
    fn write(mut self, value: DynamicMassCtx) -> Result<Self, Error> {
        self.dynamic_mass = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<DynamicMassCtx> for Context {
    fn read(&self) -> DynamicMassCtx {
        self.dynamic_mass.clone().unwrap()
    }
}
//
impl ContextWrite<TotalForceCtx> for Context {
    fn write(mut self, value: TotalForceCtx) -> Result<Self, Error> {
        self.total_force = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<TotalForceCtx> for Context {
    fn read(&self) -> TotalForceCtx {
        self.total_force.clone().unwrap()
    }
}
//
impl ContextWrite<ShearForceCtx> for Context {
    fn write(mut self, value: ShearForceCtx) -> Result<Self, Error> {
        self.shear_force = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ShearForceCtx> for Context {
    fn read(&self) -> ShearForceCtx {
        self.shear_force.clone().unwrap()
    }
}
//
impl ContextWrite<BendingMomentCtx> for Context {
    fn write(mut self, value: BendingMomentCtx) -> Result<Self, Error> {
        self.bending_moment = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<BendingMomentCtx> for Context {
    fn read(&self) -> BendingMomentCtx {
        self.bending_moment.clone().unwrap()
    }
}
//
impl ContextWrite<IcingStabCtx> for Context {
    fn write(mut self, value: IcingStabCtx) -> Result<Self, Error> {
        self.icing_stab = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<IcingStabCtx> for Context {
    fn read(&self) -> IcingStabCtx {
        self.icing_stab.clone().unwrap()
    }
}
//
impl ContextWrite<StaticMassStabCtx> for Context {
    fn write(mut self, value: StaticMassStabCtx) -> Result<Self, Error> {
        self.static_mass_stab = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<StaticMassStabCtx> for Context {
    fn read(&self) -> StaticMassStabCtx {
        self.static_mass_stab.clone().unwrap()
    }
}
//
impl ContextWrite<StabilityBalanceCtx> for Context {
    fn write(mut self, value: StabilityBalanceCtx) -> Result<Self, Error> {
        self.stability_balance = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<StabilityBalanceCtx> for Context {
    fn read(&self) -> StabilityBalanceCtx {
        self.stability_balance.clone().unwrap()
    }
}
//
impl ContextWrite<MetacentricHeightCtx> for Context {
    fn write(mut self, value: MetacentricHeightCtx) -> Result<Self, Error> {
        self.metacentric_height = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<MetacentricHeightCtx> for Context {
    fn read(&self) -> MetacentricHeightCtx {
        self.metacentric_height.clone().unwrap()
    }
}
//
impl ContextWrite<LeverDiagramCtx> for Context {
    fn write(mut self, value: LeverDiagramCtx) -> Result<Self, Error> {
        self.lever_diagram = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<LeverDiagramCtx> for Context {
    fn read(&self) -> LeverDiagramCtx {
        self.lever_diagram.clone().unwrap()
    }
}
//
impl ContextWrite<WindCtx> for Context {
    fn write(mut self, value: WindCtx) -> Result<Self, Error> {
        self.wind = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<WindCtx> for Context {
    fn read(&self) -> WindCtx {
        self.wind.clone().unwrap()
    }
}
//
impl ContextWrite<WindageCtx> for Context {
    fn write(mut self, value: WindageCtx) -> Result<Self, Error> {
        self.windage = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<WindageCtx> for Context {
    fn read(&self) -> WindageCtx {
        self.windage.clone().unwrap()
    }
}
//
impl ContextWrite<RollingPeriodCtx> for Context {
    fn write(mut self, value: RollingPeriodCtx) -> Result<Self, Error> {
        self.roll_period = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<RollingPeriodCtx> for Context {
    fn read(&self) -> RollingPeriodCtx {
        self.roll_period.clone().unwrap()
    }
}
//
impl ContextWrite<RollingAmplitudeCtx> for Context {
    fn write(mut self, value: RollingAmplitudeCtx) -> Result<Self, Error> {
        self.roll_amplitude = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<RollingAmplitudeCtx> for Context {
    fn read(&self) -> RollingAmplitudeCtx {
        self.roll_amplitude.clone().unwrap()
    }
}
//
impl ContextWrite<CriterionStabilityCtx> for Context {
    fn write(mut self, value: CriterionStabilityCtx) -> Result<Self, Error> {
        self.criterion_stability = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<CriterionStabilityCtx> for Context {
    fn read(&self) -> CriterionStabilityCtx {
        self.criterion_stability.clone().unwrap()
    }
}
//
impl ContextWrite<WheatherCtx> for Context {
    fn write(mut self, value: WheatherCtx) -> Result<Self, Error> {
        self.wheather = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<WheatherCtx> for Context {
    fn read(&self) -> WheatherCtx {
        self.wheather.clone().unwrap()
    }
}
//
impl ContextWrite<StaticAngleCtx> for Context {
    fn write(mut self, value: StaticAngleCtx) -> Result<Self, Error> {
        self.static_angle = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<StaticAngleCtx> for Context {
    fn read(&self) -> StaticAngleCtx {
        self.static_angle.clone().unwrap()
    }
}
//
impl ContextWrite<DSOAreaCtx> for Context {
    fn write(mut self, value: DSOAreaCtx) -> Result<Self, Error> {
        self.dso_area = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<DSOAreaCtx> for Context {
    fn read(&self) -> DSOAreaCtx {
        self.dso_area.clone().unwrap()
    }
}
//
impl ContextWrite<DSOMaxCtx> for Context {
    fn write(mut self, value: DSOMaxCtx) -> Result<Self, Error> {
        self.dso_max = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<DSOMaxCtx> for Context {
    fn read(&self) -> DSOMaxCtx {
        self.dso_max.clone().unwrap()
    }
}
//
impl ContextWrite<DSOTimberMaxCtx> for Context {
    fn write(mut self, value: DSOTimberMaxCtx) -> Result<Self, Error> {
        self.dso_timber_max = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<DSOTimberMaxCtx> for Context {
    fn read(&self) -> DSOTimberMaxCtx {
        self.dso_timber_max.clone().unwrap()
    }
}
//
impl ContextWrite<DSOIcingMaxCtx> for Context {
    fn write(mut self, value: DSOIcingMaxCtx) -> Result<Self, Error> {
        self.dso_icing_max = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<DSOIcingMaxCtx> for Context {
    fn read(&self) -> DSOIcingMaxCtx {
        self.dso_icing_max.clone().unwrap()
    }
}
//
impl ContextWrite<DSOAngleMaxCtx> for Context {
    fn write(mut self, value: DSOAngleMaxCtx) -> Result<Self, Error> {
        self.dso_angle_max = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<DSOAngleMaxCtx> for Context {
    fn read(&self) -> DSOAngleMaxCtx {
        self.dso_angle_max.clone().unwrap()
    }
}
//
impl ContextWrite<MinMetacentricHeightCtx> for Context {
    fn write(mut self, value: MinMetacentricHeightCtx) -> Result<Self, Error> {
        self.min_metacentric_height = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<MinMetacentricHeightCtx> for Context {
    fn read(&self) -> MinMetacentricHeightCtx {
        self.min_metacentric_height.clone().unwrap()
    }
}
//
impl ContextWrite<MetacentricHeightSubdivisionCtx> for Context {
    fn write(mut self, value: MetacentricHeightSubdivisionCtx) -> Result<Self, Error> {
        self.metacentric_height_subdivision = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<MetacentricHeightSubdivisionCtx> for Context {
    fn read(&self) -> MetacentricHeightSubdivisionCtx {
        self.metacentric_height_subdivision.clone().unwrap()
    }
}
//
impl ContextWrite<AccelerationCtx> for Context {
    fn write(mut self, value: AccelerationCtx) -> Result<Self, Error> {
        self.acceleration = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<AccelerationCtx> for Context {
    fn read(&self) -> AccelerationCtx {
        self.acceleration.clone().unwrap()
    }
}
//
impl ContextWrite<CirculationCtx> for Context {
    fn write(mut self, value: CirculationCtx) -> Result<Self, Error> {
        self.circulation = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<CirculationCtx> for Context {
    fn read(&self) -> CirculationCtx {
        self.circulation.clone().unwrap()
    }
}
//
impl ContextWrite<GrainCtx> for Context {
    fn write(mut self, value: GrainCtx) -> Result<Self, Error> {
        self.grain = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<GrainCtx> for Context {
    fn read(&self) -> GrainCtx {
        self.grain.clone().unwrap()
    }
}
//
impl ContextWrite<LoadLineCtx> for Context {
    fn write(mut self, value: LoadLineCtx) -> Result<Self, Error> {
        self.load_line = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<LoadLineCtx> for Context {
    fn read(&self) -> LoadLineCtx {
        self.load_line.clone().unwrap()
    }
}
//
impl ContextWrite<ReserveBuoyncyCtx> for Context {
    fn write(mut self, value: ReserveBuoyncyCtx) -> Result<Self, Error> {
        self.reserve_buoyncy = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ReserveBuoyncyCtx> for Context {
    fn read(&self) -> ReserveBuoyncyCtx {
        self.reserve_buoyncy.clone().unwrap()
    }
}
//
impl ContextWrite<BowBoardCtx> for Context {
    fn write(mut self, value: BowBoardCtx) -> Result<Self, Error> {
        self.bow_board = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<BowBoardCtx> for Context {
    fn read(&self) -> BowBoardCtx {
        self.bow_board.clone().unwrap()
    }
}
//
impl ContextWrite<ScrewCtx> for Context {
    fn write(mut self, value: ScrewCtx) -> Result<Self, Error> {
        self.screw = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ScrewCtx> for Context {
    fn read(&self) -> ScrewCtx {
        self.screw.clone().unwrap()
    }
}
//
impl ContextWrite<CriterionDraughtCtx> for Context {
    fn write(mut self, value: CriterionDraughtCtx) -> Result<Self, Error> {
        self.criterion_draught = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<CriterionDraughtCtx> for Context {
    fn read(&self) -> CriterionDraughtCtx {
        self.criterion_draught.clone().unwrap()
    }
}

//
impl ContextWrite<ZgCtx> for Context {
    fn write(mut self, value: ZgCtx) -> Result<Self, Error> {
        self.zg = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ZgCtx> for Context {
    fn read(&self) -> ZgCtx {
        self.zg.clone().unwrap()
    }
}
//
impl ContextWrite<DraftMarkCtx> for Context {
    fn write(mut self, value: DraftMarkCtx) -> Result<Self, Error> {
        self.draft_mark = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<DraftMarkCtx> for Context {
    fn read(&self) -> DraftMarkCtx {
        self.draft_mark.clone().unwrap()
    }
}
