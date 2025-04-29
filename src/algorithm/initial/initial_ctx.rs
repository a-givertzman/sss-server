use std::collections::HashMap;
use sal_core::error::Error;

use crate::algorithm::entities::data::ship_type::ShipType;
use crate::algorithm::entities::Bounds;
use crate::algorithm::entities::data::{loads::*, stability::{*, multipler_s::MultiplerSArray}, IcingArray, Ship, Voyage};
use crate::kernel::types::eval_result::EvalResult;

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone, Default)]
pub struct InitialCtx {
    pub ship_id: String,
    pub project_id: String,
    /// разбиение на шпации - фреймы
    pub bounds: Option<Bounds>,
    /// Текстовые данные по судну
    pub ship: Option<Ship>,
    /// Численные данные по судну
    pub ship_parameters: Option<HashMap<String, f64>>,
    /// Данные по обстановке
    pub voyage: Option<Voyage>,
    /// Данные по обледенению
    pub icing: Option<IcingArray>,
    /// Постоянная нагрузка на судно
    pub load_constant: Option<LoadConstantArray>,
    /// Переменная нагрузка на судно
    pub bulk: Option<Vec::<LoadBulkData>>,
    pub liquid: Option<Vec::<LoadLiquidData>>,
    pub unit: Option<Vec::<LoadUnitData>>,
    pub gaseous: Option<Vec::<LoadGaseousData>>,
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
}
impl InitialCtx {
    ///
    /// Struct constructor
    /// - 'ship_id' - the identifier of the ship in the database
    pub fn new(ship_id: usize, project_id: &str) -> Self {
        Self {
            ship_id: format!("{ship_id}"),
            project_id: project_id.to_owned(),
            ..Default::default()
        }
    }
    ///
    pub fn ship(&self) -> Result<Ship, Error> {
        let error = Error::new("InitialCtx", "ship");
        self
            .ship
            .as_ref()
            .ok_or(error.err("No ship")).cloned()
    }
    ///
    pub fn ship_type(&self) -> Result<ShipType, Error> {
        let error = Error::new("InitialCtx", "ship_type");
        ShipType::from_str(&self.ship()?.ship_type)
            .map_err(|e| error.pass_with("ship_type", e))
    }
    ///
    pub fn navigation_area(&self) -> Result<NavigationArea, Error> {
        let error = Error::new("InitialCtx", "navigation_area");
        NavigationArea::from_str(&self.ship()?.navigation_area)
            .map_err(|e| error.pass_with("navigation_area", e))
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
