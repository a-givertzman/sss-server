//! Промежуточные структуры для serde_json для парсинга данных груза
use super::{AssignmentType, UnitCargoType};
use crate::algorithm::entities::{data::DataArray, Bound, Position};
use api_tools::error::str_err::StrErr;
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LoadUnitData {
    /// ID груза
    pub cargo_id: usize,
    /// ID assigned
    pub assigned_id: usize,
    /// ID помещения
    pub space_id: usize,
    /// Имя груза
    pub cargo_name: String,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Тип груза судна
    pub cargo_type: UnitCargoType,
    /// масса, т
    pub mass: f64,
    /// Центр тяжести, м
    pub mass_shift: Option<Position>,
    /// Средний удельный погрузочный объем, м^3/т
    pub stowage_factor: Option<f64>,
    /// Проницаемость определяет количество, на которое груз впитывает воду
    pub permeability: Option<f64>,
    /// Обьем, м^3
    pub volume: Option<f64>,
    /// Площадь поверхности груза подвергающаяся обледенению (верхняя площадь груза)
    pub icing_area: Option<f64>,
    pub centre_of_icing_area: Option<Position>,
    /// Площадь парусности груза (площадь проекции груза на ДП судна)  
    pub windage_area: Option<f64>,
    pub centre_of_windage_area: Option<Position>,
    /// Границы груза в связанной с судном системой координат
    pub bound_x1: Option<f64>,
    pub bound_x2: Option<f64>,
    pub bound_y1: Option<f64>,
    pub bound_y2: Option<f64>,
    pub bound_z1: Option<f64>,
    pub bound_z2: Option<f64>,
}
//
impl LoadUnitData {
    //
    pub fn mass(&self, bound_x: &Bound) -> Result<f64, StrErr> {
        Ok(self.mass * self.bound_x()?.part_ratio(bound_x)?)
    }
    //
    pub fn icing_area(&self, bound_x: &Bound, bound_y: &Bound) -> Result<f64, StrErr> {
        let part_x =
            if let (Some(self_bound_x1), Some(self_bound_x2)) = (self.bound_x1, self.bound_x2) {
                Bound::new(self_bound_x1, self_bound_x2)?.part_ratio(bound_x)?
            } else {
                0.
            };
        let part_y =
            if let (Some(self_bound_y1), Some(self_bound_y2)) = (self.bound_y1, self.bound_y2) {
                Bound::new(self_bound_y1, self_bound_y2)?.part_ratio(bound_y)?
            } else {
                0.
            };
        Ok(part_x * part_y * self.icing_area.unwrap_or(0.))
    }
    //
    pub fn windage_area(&self, bound_x: &Bound, bound_z: &Bound) -> Result<f64, StrErr> {
        let part_x = self.bound_x()?.part_ratio(bound_x)?;
        let part_z =
            if let (Some(self_bound_z1), Some(self_bound_z2)) = (self.bound_z1, self.bound_z2) {
                Bound::new(self_bound_z1, self_bound_z2)?.part_ratio(bound_z)?
            } else {
                return Err(StrErr(format!(
                    "LoadUnitData windage_area error: no bound_z"
                )));
            };
        Ok(part_x * part_z * self.windage_area.unwrap_or(0.))
    }
    //
    pub fn bound_x(&self) -> Result<Bound, StrErr> {
        if let (Some(self_bound_x1), Some(self_bound_x2)) = (self.bound_x1, self.bound_x2) {
            match Bound::new(self_bound_x1, self_bound_x2) {
                Ok(data) => Ok(data),
                Err(e) => return Err(StrErr(format!("LoadUnitData bound_x error: {e}"))),
            }
        } else {
            Err(StrErr(format!("LoadUnitData bound_x error: no bounds!")))
        }
    }
}

//
/*impl std::fmt::Display for LoadUnitData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadUnitData(name:{} mass:{} general_category:{} timber:{} is_on_deck:{} container:{} bound_x:({}, {}) bound_y:({}, {}) bound_z:({}, {})
            mass_shift:({}, {}, {}) horizontal_area:{} vertical_area:{} vertical_area_shift_y:({}, {}, {}) )",
            self.name,
            self.mass.unwrap_or(0.),
            self.general_category,
            self.timber,
            self.is_on_deck,
            self.container.unwrap_or(false),
            self.bound_x1,
            self.bound_x2,
            self.bound_y1.unwrap_or(0.),
            self.bound_y2.unwrap_or(0.),
            self.bound_z1.unwrap_or(0.),
            self.bound_z2.unwrap_or(0.),
            self.mass_shift_x.unwrap_or(0.),
            self.mass_shift_y.unwrap_or(0.),
            self.mass_shift_z.unwrap_or(0.),
            self.horizontal_area.unwrap_or(0.),
            self.vertical_area.unwrap_or(0.),
            self.vertical_area_shift_x.unwrap_or(0.),
            self.vertical_area_shift_y.unwrap_or(0.),
            self.vertical_area_shift_z.unwrap_or(0.),
        )
    }
}*/
/// Массив данных по грузам
pub type LoadUnitArray = DataArray<LoadUnitData>;
//
impl LoadUnitArray {
    pub fn data(&self) -> Vec<LoadUnitData> {
        self.data.clone()
    }
}
