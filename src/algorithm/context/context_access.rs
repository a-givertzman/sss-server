use super::context::Context;
use crate::algorithm::{
    context::testing_ctx::TestingCtx,
    eval::{
        criterion::*,
        icing_timber::ctx::IcingTimberCtx,
        icing_timber_bound::ctx::IcingTimberBoundCtx,
        parameters::*,
        seakeeping::eval::{
            apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx, hitting_zones::hitting_point_ctx::HittingZonesCtx, impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx, main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx, main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx, move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx, parametric_resonant_zone::parametric_resonant_zone_ctx::ParametricResonantZoneCtx, parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx, period_excitement::period_excitement_ctx::PeriodExcitementCtx, roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx
        },
        stability::*,
        strength::*,
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
            .unwrap_or_else(|| panic!("Context.read | Id '{:?}' - is not found", id))
    }
}
