use sal_core::error::Error;
use super::{context::Context, ctx_result::CtxResult};
use crate::algorithm::{eval::{parameters::*, *}, initial::initial_ctx::InitialCtx};
///
/// Provides restricted write access to the [Context] members
pub trait ContextWrite<T> {
    fn write(self, value: T) -> CtxResult<Context, Error>;
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
impl ContextWrite<Parameters> for Context {
    fn write(mut self, value: Parameters) -> CtxResult<Self, Error> {
        self.parameters = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextReadRef<Parameters> for Context {
    fn read_ref(&self) -> &Parameters {
        self.parameters
            .as_ref()
            .unwrap()
    }
}
impl ContextParamsWrite for Context {
    fn write_params(&mut self, id: ParameterID, value: f64) {
        match &mut self.parameters {
            Some(params) => {
                params.add(id, value);
            }
            None => panic!("Context.write | Parameters - is not initialised yet, id: {:?}", id)
        };
    }
}
impl ContextParamsRead for Context {
    fn read_params(&self, id: ParameterID) -> f64 {
        let params: &Parameters  = self.read_ref();
        params.get(id).expect(&format!("Context.read | Id '{:?}' - is not found", id))
    }
}

//
//
impl ContextWrite<InitialCtx> for Context {
    fn write(mut self, value: InitialCtx) -> CtxResult<Self, Error> {
        self.initial = value;
        CtxResult::Ok(self)
    }
}
impl ContextReadRef<InitialCtx> for Context {
    fn read_ref(&self) -> &InitialCtx {
        &self.initial
    }
}
//
impl ContextWrite<StrengthAreaCtx> for Context {
    fn write(mut self, value: StrengthAreaCtx) -> CtxResult<Self, Error> {
        self.strength_area = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<StrengthAreaCtx> for Context {
    fn read(&self) -> StrengthAreaCtx {
        self.strength_area.clone().unwrap()
    }
}
//
impl ContextWrite<IcingStabCtx> for Context {
    fn write(mut self, value: IcingStabCtx) -> CtxResult<Self, Error> {
        self.icing_stab = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<IcingStabCtx> for Context {
    fn read(&self) -> IcingStabCtx {
        self.icing_stab.clone().unwrap()
    }
}
//
impl ContextWrite<IcingCtx> for Context {
    fn write(mut self, value: IcingCtx) -> CtxResult<Self, Error> {
        self.icing = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<IcingCtx> for Context {
    fn read(&self) -> IcingCtx {
        self.icing.clone().unwrap()
    }
}
//
impl ContextWrite<WettingCtx> for Context {
    fn write(mut self, value: WettingCtx) -> CtxResult<Self, Error> {
        self.wetting = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<WettingCtx> for Context {
    fn read(&self) -> WettingCtx {
        self.wetting.clone().unwrap()
    }
}
//
impl ContextWrite<LoadsCtx> for Context {
    fn write(mut self, value: LoadsCtx) -> CtxResult<Self, Error> {
        self.loads = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<LoadsCtx> for Context {
    fn read(&self) -> LoadsCtx {
        self.loads.clone().unwrap()
    }
}
//
impl ContextWrite<IcingTimberCtx> for Context {
    fn write(mut self, value: IcingTimberCtx) -> CtxResult<Self, Error> {
        self.icing_timber = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<IcingTimberCtx> for Context {
    fn read(&self) -> IcingTimberCtx {
        self.icing_timber.clone().unwrap()
    }
}
//
impl ContextWrite<BalanceCtx> for Context {
    fn write(mut self, value: BalanceCtx) -> CtxResult<Self, Error> {
        self.balance = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<BalanceCtx> for Context {
    fn read(&self) -> BalanceCtx {
        self.balance.clone().unwrap()
    }
}
//
impl ContextWrite<StabilityAreaCtx> for Context {
    fn write(mut self, value: StabilityAreaCtx) -> CtxResult<Self, Error> {
        self.stability_area = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<StabilityAreaCtx> for Context {
    fn read(&self) -> StabilityAreaCtx {
        self.stability_area.clone().unwrap()
    }
}
//
impl ContextWrite<MetacentricHeightCtx> for Context {
    fn write(mut self, value: MetacentricHeightCtx) -> CtxResult<Self, Error> {
        self.metacentric_height = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<MetacentricHeightCtx> for Context {
    fn read(&self) -> MetacentricHeightCtx {
        self.metacentric_height.clone().unwrap()
    }
}
//
impl ContextWrite<LeverDiagramCtx> for Context {
    fn write(mut self, value: LeverDiagramCtx) -> CtxResult<Self, Error> {
        self.lever_diagram = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<LeverDiagramCtx> for Context {
    fn read(&self) -> LeverDiagramCtx {
        self.lever_diagram.clone().unwrap()
    }
}
//
impl ContextWrite<WindCtx> for Context {
    fn write(mut self, value: WindCtx) -> CtxResult<Self, Error> {
        self.wind = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<WindCtx> for Context {
    fn read(&self) -> WindCtx {
        self.wind.clone().unwrap()
    }
}
//
impl ContextWrite<WindageCtx> for Context {
    fn write(mut self, value: WindageCtx) -> CtxResult<Self, Error> {
        self.windage = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<WindageCtx> for Context {
    fn read(&self) -> WindageCtx {
        self.windage.clone().unwrap()
    }
}
//
impl ContextWrite<RollingPeriodCtx> for Context {
    fn write(mut self, value: RollingPeriodCtx) -> CtxResult<Self, Error> {
        self.roll_period = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<RollingPeriodCtx> for Context {
    fn read(&self) -> RollingPeriodCtx {
        self.roll_period.clone().unwrap()
    }
}
//
impl ContextWrite<RollingAmplitudeCtx> for Context {
    fn write(mut self, value: RollingAmplitudeCtx) -> CtxResult<Self, Error> {
        self.roll_amplitude = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<RollingAmplitudeCtx> for Context {
    fn read(&self) -> RollingAmplitudeCtx {
        self.roll_amplitude.clone().unwrap()
    }
}
//
impl ContextWrite<CriterionStabilityCtx> for Context {
    fn write(mut self, value: CriterionStabilityCtx) -> CtxResult<Self, Error> {
        self.criterion_stability = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<CriterionStabilityCtx> for Context {
    fn read(&self) -> CriterionStabilityCtx {
        self.criterion_stability.clone().unwrap()
    }
}
//
impl ContextWrite<WheatherCtx> for Context {
    fn write(mut self, value: WheatherCtx) -> CtxResult<Self, Error> {
        self.wheather = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<WheatherCtx> for Context {
    fn read(&self) -> WheatherCtx {
        self.wheather.clone().unwrap()
    }
}
//
impl ContextWrite<StaticAngleCtx> for Context {
    fn write(mut self, value: StaticAngleCtx) -> CtxResult<Self, Error> {
        self.static_angle = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<StaticAngleCtx> for Context {
    fn read(&self) -> StaticAngleCtx {
        self.static_angle.clone().unwrap()
    }
}
//
impl ContextWrite<DSOAreaCtx> for Context {
    fn write(mut self, value: DSOAreaCtx) -> CtxResult<Self, Error> {
        self.dso_area = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<DSOAreaCtx> for Context {
    fn read(&self) -> DSOAreaCtx {
        self.dso_area.clone().unwrap()
    }
}
//
impl ContextWrite<DSOMaxCtx> for Context {
    fn write(mut self, value: DSOMaxCtx) -> CtxResult<Self, Error> {
        self.dso_max = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<DSOMaxCtx> for Context {
    fn read(&self) -> DSOMaxCtx {
        self.dso_max.clone().unwrap()
    }
}
//
impl ContextWrite<DSOTimberMaxCtx> for Context {
    fn write(mut self, value: DSOTimberMaxCtx) -> CtxResult<Self, Error> {
        self.dso_timber_max = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<DSOTimberMaxCtx> for Context {
    fn read(&self) -> DSOTimberMaxCtx {
        self.dso_timber_max.clone().unwrap()
    }
}
//
impl ContextWrite<DSOIcingMaxCtx> for Context {
    fn write(mut self, value: DSOIcingMaxCtx) -> CtxResult<Self, Error> {
        self.dso_icing_max = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<DSOIcingMaxCtx> for Context {
    fn read(&self) -> DSOIcingMaxCtx {
        self.dso_icing_max.clone().unwrap()
    }
}
//
impl ContextWrite<DSOAngleMaxCtx> for Context {
    fn write(mut self, value: DSOAngleMaxCtx) -> CtxResult<Self, Error> {
        self.dso_angle_max = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<DSOAngleMaxCtx> for Context {
    fn read(&self) -> DSOAngleMaxCtx {
        self.dso_angle_max.clone().unwrap()
    }
}
//
impl ContextWrite<MinMetacentricHeightCtx> for Context {
    fn write(mut self, value: MinMetacentricHeightCtx) -> CtxResult<Self, Error> {
        self.min_metacentric_height = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<MinMetacentricHeightCtx> for Context {
    fn read(&self) -> MinMetacentricHeightCtx {
        self.min_metacentric_height.clone().unwrap()
    }
}
//
impl ContextWrite<AccelerationCtx> for Context {
    fn write(mut self, value: AccelerationCtx) -> CtxResult<Self, Error> {
        self.acceleration = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<AccelerationCtx> for Context {
    fn read(&self) -> AccelerationCtx {
        self.acceleration.clone().unwrap()
    }
}
//
impl ContextWrite<CirculationCtx> for Context {
    fn write(mut self, value: CirculationCtx) -> CtxResult<Self, Error> {
        self.circulation = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<CirculationCtx> for Context {
    fn read(&self) -> CirculationCtx {
        self.circulation.clone().unwrap()
    }
}
//
impl ContextWrite<GrainCtx> for Context {
    fn write(mut self, value: GrainCtx) -> CtxResult<Self, Error> {
        self.grain = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<GrainCtx> for Context {
    fn read(&self) -> GrainCtx {
        self.grain.clone().unwrap()
    }
}
//
impl ContextWrite<LoadLineCtx> for Context {
    fn write(mut self, value: LoadLineCtx) -> CtxResult<Self, Error> {
        self.load_line = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<LoadLineCtx> for Context {
    fn read(&self) -> LoadLineCtx {
        self.load_line.clone().unwrap()
    }
}
//
impl ContextWrite<BowBoardCtx> for Context {
    fn write(mut self, value: BowBoardCtx) -> CtxResult<Self, Error> {
        self.bow_board = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<BowBoardCtx> for Context {
    fn read(&self) -> BowBoardCtx {
        self.bow_board.clone().unwrap()
    }
}
//
impl ContextWrite<BowBoScrewCtxardCtx> for Context {
    fn write(mut self, value: ScrewCtx) -> CtxResult<Self, Error> {
        self.screw = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<ScrewCtx> for Context {
    fn read(&self) -> ScrewCtx {
        self.screw.clone().unwrap()
    }
}
//
impl ContextWrite<ZgCtx> for Context {
    fn write(mut self, value: ZgCtx) -> CtxResult<Self, Error> {
        self.zg = Some(value);
        CtxResult::Ok(self)
    }
}
impl ContextRead<ZgCtx> for Context {
    fn read(&self) -> ZgCtx {
        self.zg.clone().unwrap()
    }
}







