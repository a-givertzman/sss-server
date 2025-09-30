use std::collections::HashMap;
use crate::algorithm::entities::data::ship_type::ShipType;
use crate::algorithm::entities::Bounds;
use crate::algorithm::entities::data::{loads::*, stability::{*, multipler_s::MultiplerSArray}, IcingArray, Ship, Voyage};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone, Default)]
pub struct InitialCtx {
    pub ship_id: String,
    pub project_id: String,
    /// Разбиение на теоретические шпации
    pub bounds: Option<Bounds>,
    /// Текстовые данные по судну
    pub ship: Option<Ship>,
    /// Тип судна
    pub ship_type: Option<ShipType>,
    /// Численные данные по судну
    pub ship_parameters: Option<HashMap<String, f64>>,
    /// Данные по обстановке
    pub voyage: Option<Voyage>,
    /// Район плавания судна
    pub navigation_area: Option<NavigationArea>,
    /// Данные по обледенению
    pub icing: Option<IcingArray>,
    /// Постоянная нагрузка на судно
    pub load_constant: Option<LoadConstantArray>,
    /// Переменная нагрузка на судно
    pub bulk: Option<HashMap<usize, LoadBulkData>>,
    pub liquid: Option<HashMap<usize, LoadLiquidData>>,
    pub unit: Option<Vec<LoadUnitData>>,
    pub gaseous: Option<HashMap<usize, LoadGaseousData>>,
    /// Безразмерный множитель Х_1 для расчета качки, Табл. 2.1.5.1-1
    pub multipler_x1: Option<Vec<(f64, f64)>>,
    /// Безразмерный множитель Х_2 для расчета качки, Табл. 2.1.5.1-2
    pub multipler_x2: Option<Vec<(f64, f64)>>,
    /// Безразмерный множитель S для расчета качки, Табл. 2.1.5.1-3
    pub multipler_s: Option<MultiplerSArray>,
    /// Коэффициент k для судов, имеющих скуловые кили или
    /// брусковый киль для расчета качки, Табл. 2.1.5.2
    pub coefficient_k: Option<Vec<(f64, f64)>>,
    /// Коэффициент k_theta учитывающий особенности качки судов смешанного типа
    pub coefficient_k_theta: Option<CoefficientKThetaArray>,
    /// Координаты осадок судна относительно центра
    pub load_line: Option<Vec<LoadLineParsedData>>,
    /// Высота борта на носовом перпендикуляре
    pub bow_board: Option<Vec<BowBoardParsedData>>,
    /// Координаты винтов судна относительно центра
    pub screw: Option<Vec<ScrewParsedData>>,
    /// Координаты отметок заглубления на корпусе судна
    /// относительно центра
    pub draft_mark: Option<Vec<DraftMarkParsedData>>,
    /// Минимальная допустимая метацентрическая высота деления на отсеки
    pub h_subdivision: Option<Vec<(f64, f64)>>,
}
impl InitialCtx {
    ///
    /// Struct constructor
    /// - 'ship_id' - the identifier of the ship in the database
    pub fn new(ship_id: usize, project_id: &str, bounds: Bounds) -> Self {
        Self {
            ship_id: format!("{ship_id}"),
            project_id: project_id.to_owned(),
            bounds: Some(bounds),
            ..Default::default()
        }
    }
}
// //
// //
// impl std::default::Default for InitialCtx {
//     ///
//     /// Struct constructor
//     /// - 'storage_initial_data' - [Storage] instance, where store initial data
//     fn default() -> Self {
//         Self {
//             ship_id: "NUll".to_owned(),
//             project_id: "NUll".to_owned(),
//             ..Default::default()
//         }
//     }
// }
