use sal_core::error::Error;

use super::testing_ctx::TestingCtx;
use crate::algorithm::{
    eval::{        
        criterion::*, 
        icing_timber::ctx::IcingTimberCtx,
        icing_timber_bound::ctx::IcingTimberBoundCtx,
        parameters::{IParameters, ParameterID, Parameters},
        seakeeping::eval::{
            apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx, hitting_zones::hitting_point_ctx::HittingZonesCtx, impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx, main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx, main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx, move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx, parametric_resonant_zone::parametric_resonant_zone_ctx::ParametricResonantZoneCtx, parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx, period_excitement::period_excitement_ctx::PeriodExcitementCtx, roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx
        }, 
        stability::*, 
        strength::*, *
    },
    initial::initial_ctx::InitialCtx,
};

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
            .unwrap_or_else(|| panic!("Context.read | Id '{:?}' - is not found", id))
    }
}

sal_core::define_context! {
    pub struct Context {
        parameters: Parameters,
        
        @[raw] initial: InitialCtx,
        @[opt_ref] testing: TestingCtx,
        
        unit_area: UnitAreaCtx,    
        icing_coeff: IcingCoeffCtx,
        icing_timber_bound: IcingTimberBoundCtx,
        icing_timber: IcingTimberCtx,
        impacts_high_waves: ImpactsHighWavesCtx, 
        wetting: WettingCtx,
        area_str: AreaStrCtx,
        icing_str: IcingStrCtx,
        static_mass_str: StaticMassStrCtx,
        strength_balance: StrengthBalanceCtx,
        dynamic_mass: DynamicMassCtx,
        total_force: TotalForceCtx,
        shear_force: ShearForceCtx,
        bending_moment: BendingMomentCtx,
        icing_stab: IcingStabCtx,
        static_mass_stab: StaticMassStabCtx,
        stability_balance: StabilityBalanceCtx,
        metacentric_height: MetacentricHeightCtx,
        lever_diagram: LeverDiagramCtx,
        wind: WindCtx,
        windage: WindageCtx,
        roll_period: RollingPeriodCtx,
        roll_frequency: RollingFrequencyCtx,
        hitiing_zones: HittingZonesCtx,
        roll_amplitude: RollingAmplitudeCtx,
        wheather: WheatherCtx,
        static_angle: StaticAngleCtx,
        dso_area: DSOAreaCtx,
        dso_max: DSOMaxCtx,
        dso_timber_max: DSOTimberMaxCtx,
        dso_icing_max: DSOIcingMaxCtx,
        dso_angle_max: DSOAngleMaxCtx,
        min_metacentric_height: MinMetacentricHeightCtx,
        metacentric_height_subdivision: MetacentricHeightSubdivisionCtx,
        acceleration: AccelerationCtx,
        circulation: CirculationCtx,
        grain: GrainCtx,
        criterion_stability: CriterionStabilityCtx,
        zg: ZgCtx,
        load_line: LoadLineCtx,
        reserve_buoyncy: ReserveBuoyncyCtx,
        bow_board: BowBoardCtx,
        screw: ScrewCtx,
        criterion_draught: CriterionDraughtCtx,
        draft_mark: DraftMarkCtx,
        move_broching_filter: MoveBrochingFilterCtx,        
        apparent_frequencies: ApparentFrequenciesCtx,
        period_excitement: PeriodExcitementCtx,
        main_resonant_zone: MainResonantZoneCtx,
        parametric_resonant_zone: ParametricResonantZoneCtx,
        main_resonant_zone_speed_filter: MainResonantZoneSpeedFilterCtx,
        parametric_resonant_zone_speed_filter: ParametricResonantZoneSpeedFilterCtx,
    }
}
//
//
impl Context {
    ///
    /// New instance [Context]
    /// - 'initial' - [InitialCtx] instance, where store initial data
    pub fn new(initial: InitialCtx) -> Self {
        Self {
            initial,
            ..Self::default()
        }
    }
}
