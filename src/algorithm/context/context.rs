use super::testing_ctx::TestingCtx;
use crate::algorithm::{
    eval::{        
        icing_timber::ctx::IcingTimberCtx,
        icing_timber_bound::ctx::IcingTimberBoundCtx, parameters::Parameters, 
        stability::*,
        strength::*, 
        criterion::*, 
        *,
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
    /// Диаграмма плеч статической и динамической остойчивости
    pub(super) lever_diagram: Option<LeverDiagramCtx>,
    /// Расчет плеча кренящего момента от давления ветра
    pub(super) wind: Option<WindCtx>,
    /// Парусность судна
    pub(super) windage: Option<WindageCtx>,
    /// Период собственных бортовых колебаний судна
    pub(super) roll_period: Option<RollingPeriodCtx>,
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
 //   /// Результаты расчета по прочности
  //  pub(super) result_str: Option<ResultStrCtx>,
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
