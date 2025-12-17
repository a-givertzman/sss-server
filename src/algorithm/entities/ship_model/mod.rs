mod grain_moment;
mod volume_max;
pub mod ship_model;
pub mod stability_result;

use crate::algorithm::entities::{Bounds, Moment, data::loads::{AssignmentType, LiquidCargoType}};

///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceStrengthQuery {
    ///
    pub trim: f64,
    pub draught: f64,
    /// Плотность забортной воды
    pub water_density: f64,
    /// Распределение массы судна порожнем и грузов размещенных на судне:
    /// генерального груза (unitCargoAssignment), контейнеров (containerCargoAssignment),
    /// массы обледенения и намокания;
    pub distr_static: Vec<f64>,
    /// навалочный груз
    pub bulk: Vec<BulkData>,
    /// жидкий груз
    pub liquid: Vec<LiquidData>,
    /// газообразный груз
    pub gaseous: Vec<GaseousData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>, // TODO сейчас не учитываются, добавить в расчет для отсеков
    //    /// номера поврежденных помещений, TODO - только для аварийного расчета
    //    pub damaged_compartment: Vec<String>,
    /// точность расчета
    pub epsilon: f64,
    /// шпации разбиения
    pub bounds: Bounds,
}
///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceStabilityQuery {
    /// Плотность забортной воды
    pub water_density: f64,
    /// масса судна порожнем и грузов размещенных на судне:
    /// генерального груза (unitCargoAssignment), контейнеров (containerCargoAssignment),
    /// газообразного груза (gaseousCargoAssignment), массы обледенения и намокания;
    pub mass_const: f64,
    /// Сумарный момент за вычетом смещяемых и насыпных грузов
    pub moment_const: Moment,
    /// навалочный груз
    pub bulk: Vec<BulkData>,
    /// жидкий груз
    pub liquid: Vec<LiquidData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>, // TODO сейчас не учитываются, добавить в расчет для отсеков
    /// номера поврежденных помещений, TODO - только для аварийного расчета
    pub damaged_compartment: Vec<String>,
}
///
/// Груз, для которого центр массы и распределение зависит от
/// объема и/или положения корпуса, считается в модели
/// Сыпучий груз
#[derive(Debug, Clone)]
pub struct BulkData {    
    pub assignment_id: usize,// ID assigned
    pub assigment_type: AssignmentType,  // Тип назначения груза
    pub space_id: String, // ID помещения
    pub mass: f64,
    pub volume: f64,
    pub shiftable: bool,
}
///
/// Жидкий груз
#[derive(Debug, Clone)]
pub struct LiquidData {
    pub assignment_id: usize,// ID assigned
    pub assigment_type: AssignmentType,  // Тип назначения груза
    pub space_id: String, // ID помещения    
    pub cargo_type: LiquidCargoType, // Тип жидкого груза
    pub use_max_moment: bool, // Признак использования максимального значения момента свободной поверхности жидкости
    pub is_cargo_tank: bool,
    pub mass: f64,
    pub volume: f64,
    pub density: f64,
}
///
/// Газообразный груз
/// Для него распределение не зависит от обьема или массы,
/// считаем распределение по отсеку при максимальном объеме отсека
#[derive(Debug, Clone)]
pub struct GaseousData {
    pub assignment_id: usize, // ID assigned
    pub assigment_type: AssignmentType,  // Тип назначения груза
    pub space_id: String, // ID помещения
    pub mass: f64,
}
/// Разбиение площадей поверхности корпуса по шпациям для расчета прочности
#[derive(Debug, Clone)]
pub struct StrengthArea {
    pub v: Vec<f64>,
    pub h: Vec<f64>,
}
/// Площади и моменты поверхности корпуса для расчета остойчивости
#[derive(Debug, Clone)]
pub struct StabilityArea {
    /// Площадь парусности сплошных поверхностей для осадки d_min без палубного груза, м^2
    pub av_cs_dmin1: f64,
    /// Cтатический момент площади парусности по длине относительно начала координат, м^2
    pub mv_x_cs_dmin1: f64,
    /// Cтатический момент площади парусности по высоте относительно ОП, м^2
    pub mv_z_cs_dmin1: f64,
    /// Разница в площадях парусности для текущей осадки и осадки d_min, м^2
    pub delta_av: f64,
    /// Разница в статических моментах для текущей осадки и осадки dmin относительно начала координат, м^3
    pub delta_mv_x: f64,
    /// Разница в статических моментах для текущей осадки и осадки dmin относительно ОП, м^3
    pub delta_mv_z: f64,
    /// Отстояние по вертикали центра площади проекции подводной части корпуса на диаметральную плоскость 
    /// в прямом положении судна (при нулевом крене) на спокойной воде для текущей осадки [м]
    pub area_volume_z: f64,    
    /// Площадь горизонтальных поверхностей судна
    pub area_horisontal: f64,
    /// Положение центра площади горизонтальных поверхностей по оси Z относительно опорной плоскости 
    pub area_horisontal_z: f64,
}