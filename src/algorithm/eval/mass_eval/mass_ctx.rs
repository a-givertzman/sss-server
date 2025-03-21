use crate::algorithm::entities::{area::HAreaStrength, data::strength::VerticalArea};
///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct MassCtx {
    /// Постоянная масса судна распределенная по шпациям
    loads_const: Rc<Vec<Rc<LoadMass>>>,
    /// Учет распределения обледенения судна
    icing_mass: Rc<dyn IIcingMass>,
    /// Учет намокания палубного груза - леса
    wetting_mass: Rc<dyn IWettingMass>,
    /// Все грузы судна
    loads_variable: Rc<Vec<Rc<LoadMass>>>,
    /// Вектор разбиения на отрезки для эпюров
    bounds: Rc<Bounds>,
    /// Набор результатов расчетов для записи в БД
    results: Rc<dyn IResults>,
    parameters: Rc<dyn IParameters>,
    /// Вектор разбиения грузов по отрезкам
    // TODO - закешировать разбиение грузов  с коэффициентами
    // по отрезкам
    //   bounds_values: Rc<RefCell<Option<Vec<(<Rc<LoadMass>>)>>>>,
    /// Суммарная масса балласта
    ballast: Rc<RefCell<Option<f64>>>,
    /// Суммарная масса запасов
    stores: Rc<RefCell<Option<f64>>>,
    /// Суммарная масса обледенения
    icing: Rc<RefCell<Option<f64>>>,
    /// Суммарная масса намокания
    wetting: Rc<RefCell<Option<f64>>>,
    /// Суммарная масса груза
    cargo: Rc<RefCell<Option<f64>>>,
    /// Суммарная масса зерновых перегородок
    bulkhead: Rc<RefCell<Option<f64>>>,
    /// Суммарная масса корпуса
    lightship: Rc<RefCell<Option<f64>>>,
    /// Суммарная масса
    sum: Rc<RefCell<Option<f64>>>,
    /// Распределение массы по вектору разбиения
    mass_values: Rc<RefCell<Option<Vec<f64>>>>,
}
