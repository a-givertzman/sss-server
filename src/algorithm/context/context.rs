use super::testing_ctx::TestingCtx;
use crate::algorithm::{
    eval::{        
        criterion::*, icing_timber::ctx::IcingTimberCtx, icing_timber_bound::ctx::IcingTimberBoundCtx, parameters::Parameters, room_element_report::room_element_report_ctx::RoomElementReportCtx, seakeeping::eval::{
            apparent_frequencies::apparent_frequencies_ctx::ApparentFrequenciesCtx, hitting_zones::hitting_point_ctx::HittingZonesCtx, impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx, main_resonant_zone::main_resonant_zone_ctx::MainResonantZoneCtx, main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx, move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx, parametric_resonant_zone::parametric_resonant_zone_ctx::ParametricResonantZoneCtx, parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx, period_excitement::period_excitement_ctx::PeriodExcitementCtx, roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx
        }, stability::*, strength::*, *
    },
    initial::initial_ctx::InitialCtx,
};
///
/// # Calculation context
/// - Provides read/write access to initial
/// - R/W access to the isoleted data of each step of computations
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// where store [initial data](design\docs\algorithm\part01\initial_data.md)
    pub(super) initial: InitialCtx,
    /// Массив [кажущихся частот волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
    pub(super) apparent_frequencies: Option<ApparentFrequenciesCtx>,
    // Результаты расчета в виде (id, value)
    // id в соответствии с https://github.com/a-givertzman/sss/blob/35-shipmodel-fix-unit-cargo/docs/user-guide/ru/part08_stability/chapter03_parametresStability.md
    pub(super) parameters: Parameters,
    /// Площади парусности палубных грузов для расчета остойчивости
    pub(super) unit_area: Option<UnitAreaCtx>,    
    /// Коэффициенты для расчета обледенения судна
    pub(super) icing_coeff: Option<IcingCoeffCtx>,
    /// Ограничение горизонтальной площади обледенения палубного груза - леса
    pub(super) icing_timber_bound: Option<IcingTimberBoundCtx>,
    /// Площади обледенения горизонтальных поверхностей палубного лесного груза
    pub(super) icing_timber: Option<IcingTimberCtx>,
    /// Массив скоростей движения, при которых происходит явление последовательных ударов высоких волн
    pub(super) impacts_high_waves: Option<ImpactsHighWavesCtx>, 
    /// Учет намокания груза
    pub(super) wetting: Option<WettingCtx>,
    /// Распределение площади для расчета прочности
    pub(super) area_str: Option<AreaStrCtx>,
    /// Учет обледенения судна и груза для прочности
    pub(super) icing_str: Option<IcingStrCtx>,
    /// Расчет массы корпуса и статических грузов судна для прочности
    pub(super) static_mass_str: Option<StaticMassStrCtx>,
    /// Расчет равновесного положения судна для прочности
    pub(super) strength_balance: Option<StrengthBalanceCtx>,
    /// Распределение массы смещаемых грузов судна для прочности
    pub(super) dynamic_mass: Option<DynamicMassCtx>,
    /// Результирующая нагрузка на шпацию
    pub(super) total_force: Option<TotalForceCtx>,
    /// Срезающая сила, действующая на корпус судна
    pub(super) shear_force: Option<ShearForceCtx>,
    /// Изгибающий момент
    pub(super) bending_moment: Option<BendingMomentCtx>,
    /// Учет обледенения судна и груза для остойчивости
    pub(super) icing_stab: Option<IcingStabCtx>,
    /// Расчет массы корпуса и статических грузов судна
    pub(super) static_mass_stab: Option<StaticMassStabCtx>,
    /// Расчет равновесного положения судна для остойчивости
    pub(super) stability_balance: Option<StabilityBalanceCtx>,
    /// Исправленная метацентрическая высота
    pub(super) metacentric_height: Option<MetacentricHeightCtx>,
    /// Массив скоростей хода, уз, при которых возникает движение судна на гребне волны и брочинг
    pub(super) move_broching_filter: Option<MoveBrochingFilterCtx>,
    /// Диаграмма плеч статической и динамической остойчивости
    pub(super) lever_diagram: Option<LeverDiagramCtx>,
    /// Расчет плеча кренящего момента от давления ветра
    pub(super) wind: Option<WindCtx>,
    /// Парусность судна
    pub(super) windage: Option<WindageCtx>,
    /// Период собственных бортовых колебаний судна
    pub(super) roll_period: Option<RollingPeriodCtx>,
    /// Частоты собственных бортовых колебаний судна
    pub(super) roll_frequency: Option<RollingFrequencyCtx>,
    /// Расчет пересечения с зонами резонанса
    pub(super) hitiing_zones: Option<HittingZonesCtx>,
    /// Амплитуда качки судна  
    pub(super) roll_amplitude: Option<RollingAmplitudeCtx>,
    /// Критерий погоды К
    pub(super) wheather: Option<WheatherCtx>,
    /// Статический угол крена от действия постоянного ветра
    pub(super) static_angle: Option<StaticAngleCtx>,
    /// Критерий площади под диаграммой статической остойчивости
    pub(super) dso_area: Option<DSOAreaCtx>,
    /// Критерий максимум диаграммы статической остойчивости
    pub(super) dso_max: Option<DSOMaxCtx>,
    /// Критерий максимум диаграммы статической остойчивости для лесовозов
    pub(super) dso_timber_max: Option<DSOTimberMaxCtx>,
    /// Критерий максимум диаграммы статической остойчивости с учетом обледенения
    pub(super) dso_icing_max: Option<DSOIcingMaxCtx>,
    /// Угол, соответствующий максимуму диаграммы статической остойчивости
    pub(super) dso_angle_max: Option<DSOAngleMaxCtx>,
    /// Критерий минимальной метацентрической высоты
    pub(super) min_metacentric_height: Option<MinMetacentricHeightCtx>,
    /// Критерий метацентрической высоты
    pub(super) metacentric_height_subdivision: Option<MetacentricHeightSubdivisionCtx>,
    /// Параметрическая зона резонансной бортовой качки
    pub(super) parametric_resonant_zone: Option<ParametricResonantZoneCtx>,
    /// Массив скоростей хода, при которых кажущаяся частота волнения находится в диапазоне параметрического резонанса частот
    pub(super) parametric_resonant_zone_speed_filter: Option<ParametricResonantZoneSpeedFilterCtx>,
    /// Период волнения
    pub(super) period_exctiment: Option<PeriodExcitementCtx>,
    /// Критерий ускорения 𝐾∗
    pub(super) acceleration: Option<AccelerationCtx>,
    /// Критерий крена на циркуляции
    pub(super) circulation: Option<CirculationCtx>,
    /// Критерий при перевозки навалочных смещаемых грузов
    pub(super) grain: Option<GrainCtx>,
    /// Критерии проверки остойчивости судна
    pub(super) criterion_stability: Option<CriterionStabilityCtx>,
    /// Результаты для ZG
    pub(super) zg: Option<ZgCtx>,
    /// Критерий осадки по грузовой марке
    pub(super) load_line: Option<LoadLineCtx>,
    /// Критерий запаса плавучести в носу
    pub(super) reserve_buoyncy: Option<ReserveBuoyncyCtx>,
    /// Критерий высоты на носовом перпендикуляре
    pub(super) bow_board: Option<BowBoardCtx>,
    /// Расчет критерия заглубления винта
    pub(super) screw: Option<ScrewCtx>,
    /// Критерии проверки посадки судна
    pub(super) criterion_draught: Option<CriterionDraughtCtx>,
    /// Расчет уровня заглубления для координат отметок заглубления на корпусе судна
    pub(super) draft_mark: Option<DraftMarkCtx>,
    /// Период волнения Tw в секундах в диапазоне от 1.0 до 15.0 секунд, с шагом 0.1 секунда
    pub(super) period_excitement: Option<PeriodExcitementCtx>,
    /// Основная зона резонансной бортовой качки
    pub(super) main_resonant_zone: Option<MainResonantZoneCtx>,
    /// Массив скоростей хода, при которых кажущаяся частота волнения находится в диапазоне основого резонанса частот
    pub(super) main_resonant_zone_speed_filter: Option<MainResonantZoneSpeedFilterCtx>,
    /// Сформированный отчёт по "Элементы помещений"
    pub(super) room_element_report: Option<RoomElementReportCtx>,
    ///
    /// Uset for testing only
    #[allow(dead_code)]
    pub testing: Option<TestingCtx>,
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
