use sal_core::error::Error;
use super::context::Context;
use crate::algorithm::{
    eval::{
        apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx, 
        impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx, 
        import_model::{
            convert_diametrical_buttocks_to_trimesh::convert_diametrical_buttocks_to_trimesh_ctx::ConvertDiametricalButtocksToTrimeshCtx, convert_surface_outer_to_trimesh::convert_surface_outer_to_trimesh_ctx::ConvertSurfaceOuterToTrimeshCtx, convert_surface_superstructure_to_trimesh::convert_surface_superstructure_to_trimesh_ctx::ConvertSurfaceSuperStructureToTrimeshCtx, import_model_initial_points::import_model_initial_points_ctx::ImportModelInitialPointsCtx
        },
        import_tanks::{
            convert_tanks_to_trimesh_ctx::ConvertTanksToTrimeshCtx, 
            import_3d_tanks_ctx::Import3DTanksCtx
        }, 
        main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx, 
        main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx, 
        move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx, parameters::*, 
        parametric_resonant_zone::parametric_resonant_zone_ctx::ParametricResonantZoneCtx, 
        parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx, 
        period_excitement::period_excitement_ctx::PeriodExcitementCtx,
        roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx,
        vessel_max_speed::vessel_max_speed_ctx::VesselMaxSpeedCtx, *
    }, 
    initial::initial_ctx::InitialCtx
};
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
impl ContextWrite<Parameters> for Context {
    fn write(mut self, value: Parameters) -> Result<Self, Error> {
        self.parameters = Some(value);
        Result::Ok(self)
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
//
impl ContextWrite<ApparentFrequenciesCtx> for Context {
    fn write(mut self, value: ApparentFrequenciesCtx) -> Result<Self, Error> {
        self.apparent_frequencies = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ApparentFrequenciesCtx> for Context {
    fn read(&self) -> ApparentFrequenciesCtx {
        self.apparent_frequencies.clone().unwrap()
    }
}
//
impl ContextWrite<StrengthAreaCtx> for Context {
    fn write(mut self, value: StrengthAreaCtx) -> Result<Self, Error> {
        self.strength_area = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<StrengthAreaCtx> for Context {
    fn read(&self) -> StrengthAreaCtx {
        self.strength_area.clone().unwrap()
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
impl ContextWrite<IcingCtx> for Context {
    fn write(mut self, value: IcingCtx) -> Result<Self, Error> {
        self.icing = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<IcingCtx> for Context {
    fn read(&self) -> IcingCtx {
        self.icing.clone().unwrap()
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
impl ContextWrite<VesselMaxSpeedCtx> for Context {
    fn write(mut self, value: VesselMaxSpeedCtx) -> Result<Self, Error> {
        self.vmax = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<VesselMaxSpeedCtx> for Context {
    fn read(&self) -> VesselMaxSpeedCtx {
        self.vmax.clone().unwrap()
    }
}
//
impl ContextWrite<LoadsCtx> for Context {
    fn write(mut self, value: LoadsCtx) -> Result<Self, Error> {
        self.loads = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<LoadsCtx> for Context {
    fn read(&self) -> LoadsCtx {
        self.loads.clone().unwrap()
    }
}
//
impl ContextWrite<MainResonantZoneCtx> for Context {
    fn write(mut self, value: MainResonantZoneCtx) -> Result<Self, Error> {
        self.main_resonant_zone = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<MainResonantZoneCtx> for Context {
    fn read(&self) -> MainResonantZoneCtx {
        self.main_resonant_zone.clone().unwrap()
    }
}
//
impl ContextWrite<MainResonantZoneSpeedFilterCtx> for Context {
    fn write(mut self, value: MainResonantZoneSpeedFilterCtx) -> Result<Self, Error> {
        self.main_resonant_zone_speed_filter = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<MainResonantZoneSpeedFilterCtx> for Context {
    fn read(&self) -> MainResonantZoneSpeedFilterCtx {
        self.main_resonant_zone_speed_filter.clone().unwrap()
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
impl ContextWrite<ImpactsHighWavesCtx> for Context {
    fn write(mut self, value: ImpactsHighWavesCtx) -> Result<Self, Error> {
        self.impacts_high_waves = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ImpactsHighWavesCtx> for Context {
    fn read(&self) -> ImpactsHighWavesCtx {
        self.impacts_high_waves.clone().unwrap()
    }
}
//
impl ContextWrite<BalanceCtx> for Context {
    fn write(mut self, value: BalanceCtx) -> Result<Self, Error> {
        self.balance = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<BalanceCtx> for Context {
    fn read(&self) -> BalanceCtx {
        self.balance.clone().unwrap()
    }
}
//
impl ContextWrite<StabilityAreaCtx> for Context {
    fn write(mut self, value: StabilityAreaCtx) -> Result<Self, Error> {
        self.stability_area = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<StabilityAreaCtx> for Context {
    fn read(&self) -> StabilityAreaCtx {
        self.stability_area.clone().unwrap()
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
impl ContextWrite<MoveBrochingFilterCtx> for Context {
    fn write(mut self, value: MoveBrochingFilterCtx) -> Result<Self, Error> {
        self.move_broching_filter = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<MoveBrochingFilterCtx> for Context {
    fn read(&self) -> MoveBrochingFilterCtx {
        self.move_broching_filter.clone().unwrap()
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
impl ContextWrite<RollingFrequencyCtx> for Context {
    fn write(mut self, value: RollingFrequencyCtx) -> Result<Self, Error> {
        self.roll_frequency = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<RollingFrequencyCtx> for Context {
    fn read(&self) -> RollingFrequencyCtx {
        self.roll_frequency.clone().unwrap()
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
impl ContextWrite<ParametricResonantZoneCtx> for Context {
    fn write(mut self, value: ParametricResonantZoneCtx) -> Result<Self, Error> {
        self.parametric_resonant_zone = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ParametricResonantZoneCtx> for Context {
    fn read(&self) -> ParametricResonantZoneCtx {
        self.parametric_resonant_zone.clone().unwrap()
    }
}
//
impl ContextWrite<ParametricResonantZoneSpeedFilterCtx> for Context {
    fn write(mut self, value: ParametricResonantZoneSpeedFilterCtx) -> Result<Self, Error> {
        self.parametric_resonant_zone_speed_filter = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ParametricResonantZoneSpeedFilterCtx> for Context {
    fn read(&self) -> ParametricResonantZoneSpeedFilterCtx {
        self.parametric_resonant_zone_speed_filter.clone().unwrap()
    }
}
//
impl ContextWrite<PeriodExcitementCtx> for Context {
    fn write(mut self, value: PeriodExcitementCtx) -> Result<Self, Error> {
        self.period_exctiment = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<PeriodExcitementCtx> for Context {
    fn read(&self) -> PeriodExcitementCtx {
        self.period_exctiment.clone().unwrap()
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
//
impl ContextWrite<ImportModelInitialPointsCtx> for Context {
    fn write(mut self, value: ImportModelInitialPointsCtx) -> Result<Self, Error> {
        self.import_3d_model = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ImportModelInitialPointsCtx> for Context {
    fn read(&self) -> ImportModelInitialPointsCtx {
        self.import_3d_model.clone().unwrap()
    }
}
//
impl ContextWrite<ConvertSurfaceOuterToTrimeshCtx> for Context {
    fn write(mut self, value: ConvertSurfaceOuterToTrimeshCtx) -> Result<Self, Error> {
        self.converted_surface_outer = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ConvertSurfaceOuterToTrimeshCtx> for Context {
    fn read(&self) -> ConvertSurfaceOuterToTrimeshCtx {
        self.converted_surface_outer.clone().unwrap()
    }
}
//
impl ContextWrite<Import3DTanksCtx> for Context {
    fn write(mut self, value: Import3DTanksCtx) -> Result<Self, Error> {
        self.import_3d_tanks = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<Import3DTanksCtx> for Context {
    fn read(&self) -> Import3DTanksCtx {
        self.import_3d_tanks.clone().unwrap()
    }
}
//
impl ContextWrite<ConvertTanksToTrimeshCtx> for Context {
    fn write(mut self, value: ConvertTanksToTrimeshCtx) -> Result<Self, Error> {
        self.converted_3d_tanks = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ConvertTanksToTrimeshCtx> for Context {
    fn read(&self) -> ConvertTanksToTrimeshCtx {
        self.converted_3d_tanks.clone().unwrap()
    }
}
//
impl ContextWrite<ConvertSurfaceSuperStructureToTrimeshCtx> for Context {
    fn write(mut self, value: ConvertSurfaceSuperStructureToTrimeshCtx) -> Result<Self, Error> {
        self.converted_surface_superstructure = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ConvertSurfaceSuperStructureToTrimeshCtx> for Context {
    fn read(&self) -> ConvertSurfaceSuperStructureToTrimeshCtx {
        self.converted_surface_superstructure.clone().unwrap()
    }
}
//
impl ContextWrite<ConvertDiametricalButtocksToTrimeshCtx> for Context {
    fn write(mut self, value: ConvertDiametricalButtocksToTrimeshCtx) -> Result<Self, Error> {
        self.converted_diametrical_buttocks = Some(value);
        Result::Ok(self)
    }
}
impl ContextRead<ConvertDiametricalButtocksToTrimeshCtx> for Context {
    fn read(&self) -> ConvertDiametricalButtocksToTrimeshCtx {
        self.converted_diametrical_buttocks.clone().unwrap()
    }
}






