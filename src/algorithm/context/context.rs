use crate::
    algorithm::{eval::*, initial::initial_ctx::InitialCtx
    }
;
use super::testing_ctx::TestingCtx;
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
