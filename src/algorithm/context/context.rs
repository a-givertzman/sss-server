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
