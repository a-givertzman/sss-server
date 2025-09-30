//! Промежуточные структуры для serde_json для парсинга данных груза
use super::{AssignmentType, UnitCargoType};
use crate::algorithm::entities::{Bound, Moment, Position, data::DataArray};
use sal_core::error::Error;
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LoadUnitData {
    /// ID груза
    pub cargo_id: usize,
    /// ID assigned
    pub assigned_id: usize,
    /// ID помещения
    pub space_id: String,
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
    pub fn mass(&self, bound_x: &Bound) -> Result<f64, Error> {
        Ok(self.mass
            * self
                .bound_x()
                .map_err(|e| Error::new("LoadUnitData", "mass").pass_with("bound_x", e))?
                .part_ratio(bound_x)
                .map_err(|e| Error::new("LoadUnitData", "mass").pass_with("part_ratio", e))?)
    }
    /// Расчет площади обледенения по заданным ограничениям.
    /// Возвращает площадь, попадающую в ограничение, момент плозади и дельту момента площади относительно палубы (bound_z1)
    pub fn icing_area(
        &self,
        bound_x: &Bound,
        bound_y: &Bound,
    ) -> Result<(f64, Moment, Moment), Error> {
        let error = Error::new("LoadUnitData", "icing_area");
        let self_bound_x =
            if let (Some(self_bound_x1), Some(self_bound_x2)) = (self.bound_x1, self.bound_x2) {
                Bound::new(self_bound_x1, self_bound_x2)
                    .map_err(|e| error.pass_with("self_bound_x", e))?
            } else {
                return Err(Error::new("LoadUnitData", "icing_area error: no _bound_x"));
            };
        let self_bound_y =
            if let (Some(self_bound_y1), Some(self_bound_y2)) = (self.bound_y1, self.bound_y2) {
                Bound::new(self_bound_y1, self_bound_y2)
                    .map_err(|e| error.pass_with("self_bound_y", e))?
            } else {
                return Err(error.err("no self_bound_y"));
            };
        let part_x = self_bound_x.part_ratio(bound_x)
            .map_err(|e| error.pass_with("part_x part_ratio", e))?;
        let part_y = self_bound_y.part_ratio(bound_y)
            .map_err(|e| error.pass_with("part_y part_ratio", e))?;
        let area = part_x * part_y * self.icing_area.unwrap_or(0.);
        let (full_moment, delta_moment) = if area > 0. {
            let center_x = self_bound_x.intersect(bound_x)
                .map_err(|e| error.pass_with("center_x intersect", e))?
                .center().unwrap_or(0.);
            let center_y = self_bound_y.intersect(bound_y)
                .map_err(|e| error.pass_with("center_y intersect", e))?
                .center().unwrap_or(0.);
            let center_z = self.centre_of_icing_area.unwrap_or(Position::zero()).z();
            let delta_z = (self.bound_z2.unwrap_or(0.) - self.bound_z1.unwrap_or(0.)).max(0.);
            (
                Moment::from_pos(Position::new(center_x, center_y, center_z), area),
                Moment::from_pos(Position::new(center_x, center_y, delta_z), area),
            )
        } else {
            (Moment::zero(), Moment::zero())
        };
        Ok((area, full_moment, delta_moment))
    }
    //
    pub fn windage_area(&self, bound_x: &Bound, bound_z: &Bound) -> Result<f64, Error> {
        let error = Error::new("LoadUnitData", "windage_area");
        let part_x = self.bound_x()
            .map_err(|e| error.pass_with("part_x self.bound_x", e))?
            .part_ratio(bound_x)
            .map_err(|e| error.pass_with("part_x part_ratio", e))?;
        let part_z =
            if let (Some(self_bound_z1), Some(self_bound_z2)) = (self.bound_z1, self.bound_z2) {
                Bound::new(self_bound_z1, self_bound_z2)
                    .map_err(|e| error.pass_with("part_z Bound::new", e))?
                    .part_ratio(bound_z)
                    .map_err(|e| error.pass_with("part_x part_ratio", e))?
            } else {
                return Err(Error::from("LoadUnitData.windage_area | no bound_z"));
            };
        Ok(part_x * part_z * self.windage_area.unwrap_or(0.))
    }
    //
    pub fn bound_x(&self) -> Result<Bound, Error> {
        let error = Error::new("LoadUnitData", "bound_x");
        if let (Some(self_bound_x1), Some(self_bound_x2)) = (self.bound_x1, self.bound_x2) {
            match Bound::new(self_bound_x1, self_bound_x2) {
                Ok(data) => Ok(data),
                Err(e) => return Err(error.pass(e)),
            }
        } else {
            Err(error.err("no bounds!"))
        }
    }
    //
    pub fn mass_shift(&self) -> Result<Position, Error> {
        let error = Error::new("LoadUnitData", "mass_shift");
        if let Some(mass_shift) = self.mass_shift {
            Ok(mass_shift)
        } else {
            if let Ok(bound_x) = self.bound_x() {
                if let (
                    Some(center_x),
                    Some(bound_y1),
                    Some(bound_y2),
                    Some(bound_z1),
                    Some(bound_z2),
                ) = (
                    bound_x.center(),
                    self.bound_y1,
                    self.bound_y2,
                    self.bound_z1,
                    self.bound_z2,
                ) {
                    let center_y = bound_y1 + (bound_y2 - bound_y1) / 2.;
                    let center_z = bound_z1 + (bound_z2 - bound_z1) / 2.;
                    return Ok(Position::new(center_x, center_y, center_z));
                }
            }
            Err(error.err("no mass_shift and bounds!"))
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
    pub fn data(self) -> Vec<LoadUnitData> {
        self.data.into_iter().filter(|v| v.mass > 0.).collect()
    }
}
