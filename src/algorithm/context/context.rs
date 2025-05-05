use super::testing_ctx::TestingCtx;
use crate::algorithm::{
    eval::{parameters::Parameters, *},
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
    // Результаты расчета в виде (id, value)
    // id в соответствии с https://github.com/a-givertzman/sss/blob/35-shipmodel-fix-unit-cargo/docs/user-guide/ru/part08_stability/chapter03_parametresStability.md
    pub(super) parameters: Option<Parameters>,
    /// Распределение площади для расчета прочности
    pub(super) strength_area: Option<StrengthAreaCtx>,
    /// Коэффициенты для расчета обледенения судна
    pub(super) icing_stab: Option<IcingStabCtx>,
    /// Ограничение горизонтальной площади обледенения палубного груза - леса
    pub(super) icing_timber: Option<IcingTimberCtx>,
    /// Учет обледенения судна и  груза
    pub(super) icing: Option<IcingCtx>,
    /// Учет намокания груза
    pub(super) wetting: Option<WettingCtx>,
    /// Все грузы судна
    pub(super) loads: Option<LoadsCtx>,
    /// Расчет равновесного положения судна
    /// Параметры + данные по смещаемым грузам
    pub(super) balance: Option<BalanceCtx>,
    /// Площади горизонтальных поверхностей и
    /// площади парусности судна для расчета остойчивости
    pub(super) stability_area: Option<StabilityAreaCtx>,
    /// Исправленная метацентрическая высота
    pub(super) metacentric_height: Option<MetacentricHeightCtx>,
    /// Диаграмма плеч статической и динамической остойчивости
    pub(super) lever_diagram: Option<LeverDiagramCtx>,
    /// Расчет плеча кренящего момента от давления ветра
    pub(super) wind: Option<WindCtx>,
    /// Парусность судна
    pub(super) windage: Option<WindageCtx>,
    /// Период качки судна  
    pub(super) roll_period: Option<RollingPeriodCtx>,
    /// Амплитуда качки судна  
    pub(super) roll_amplitude: Option<RollingAmplitudeCtx>,
    /// Критерии проверки остойчивости судна
    pub(super) criterion_stability: Option<CriterionStabilityCtx>,
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
    /// Критерий ускорения 𝐾∗
    pub(super) acceleration: Option<AccelerationCtx>,
    /// Критерий крена на циркуляции
    pub(super) circulation: Option<CirculationCtx>,
    /// Критерий при перевозки навалочных смещаемых грузов
    pub(super) grain: Option<GrainCtx>,
    /// Критерий осадки по грузовой марке
    pub(super) load_line: Option<LoadLineCtx>,
    /// Критерий высоты на носовом перпендикуляре
    pub(super) bow_board: Option<BowBoardCtx>,
    /// Результаты для ZG
    pub(super) zg: Option<ZgCtx>,
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
