//! Нагрузка на судно: постоянный и переменный груз.
use std::{cell::RefCell, rc::Rc};

mod bulk;
mod desk;
mod mass;
mod tank;

pub use bulk::*;
pub use desk::*;
pub use mass::*;
pub use tank::*;

use crate::kernel::error::error::Error;

use super::{data::loads::{AssignmentType, CargoType, LoadBulkData, LoadConstantData, LoadGaseousData, LoadLiquidData, LoadUnitData}, *};


type Shell<T> = Rc<RefCell<Option<Rc<Vec<Rc<T>>>>>>;

/// Тип груза
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LoadingType {
    Constant,
    Ballast,
    Stores,
    CargoLoad,
    Unspecified,
}
//
impl std::fmt::Display for LoadingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LoadingType::Constant => "Constant",
                LoadingType::Ballast => "Ballast",
                LoadingType::Stores => "Stores",
                LoadingType::CargoLoad => "CargoLoad",
                LoadingType::Unspecified => "Unspecified",
            },
        )
    }
}
//
impl From<AssignmentType> for LoadingType {
    fn from(value: AssignmentType) -> Self {
        match value {
            AssignmentType::Ballast => LoadingType::Ballast,
            AssignmentType::Stores => LoadingType::Stores,
            AssignmentType::CargoLoad => LoadingType::CargoLoad,
            AssignmentType::Unspecified => LoadingType::Unspecified,
        }
    }
}
/// Абстрактный груз: контейнер, трюм или бак.
/// Имеет массу и может вернуть какая его часть попадает в указанные границы
pub trait ILoad {
    /// Суммарная масса груза
    fn mass(&self) -> f64;
    /// Границы груза вдоль продольной оси
    fn bound_x(&self) -> Bound;
    /// Смещение центра груза относительно начала координат судна
    fn shift(&self) -> Position;
}
/// Нагрузка судна: грузы, корпус, механизмы
pub struct Loads {
    load_constants: Vec<LoadConstantData>,
    shift_const: Position,
    bulk: Vec<LoadBulkData>,
    liquid: Vec<LoadLiquidData>,
    unit: Vec<LoadUnitData>,
    gaseous: Vec<LoadGaseousData>
}
//
impl Loads {
    /// Основной конструктор
    /// * load_constants - Постоянная нагрузка на судно
    /// * shift_const - Смещение центра масс постоянной нагрузки на судно
    /// * cargoes - Нагрузка судна без жидких грузов
    /// * compartments - Нагрузка судна: цистерны и трюмы
    pub fn new(
        load_constants: &'a Vec<LoadConstantData>,
        shift_const: Position,
        cargoes: &'a Vec<LoadCargo>,
        compartments: &'a Vec<CompartmentData>,
    ) -> Loads<'a> {
        Loads {
            load_constants,
            shift_const,
            cargoes,
            compartments,
            tanks: Rc::new(RefCell::new(None)),
            desks: Rc::new(RefCell::new(None)),
            bulks: Rc::new(RefCell::new(None)),
            load_variable: Rc::new(RefCell::new(None)),
            load_timber: Rc::new(RefCell::new(None)),
            loads_const: Rc::new(RefCell::new(None)),
        }
    }
    // Ленивый расчет
    fn create(&self) -> Result<(), Error> {
        let mut tanks: Vec<Rc<dyn ITank>> = Vec::new();
        let mut desks: Vec<Rc<dyn IDesk>> = Vec::new();
        let mut bulks: Vec<Rc<dyn IBulk>> = Vec::new();
        let mut load_variable: Vec<Rc<LoadMass>> = Vec::new();
        let mut load_timber: Vec<Rc<LoadMass>> = Vec::new();
        let mut loads_const: Vec<Rc<LoadMass>> = Vec::new();

        for v in self.load_constants.iter() {
            let bound_x = Bound::new(v.bound_x1, v.bound_x2)?;
            let load = Rc::new(LoadMass::new(
                v.mass,
                bound_x,
                Some(self.shift_const),
                LoadingType::from(v.loading_type),
            )?);
            log::trace!("\t Mass loads_const from load_constants:{:?} ", load);
            loads_const.push(load);
        }

        for v in self.cargoes.iter() {
            let mass_shift = if let (Some(mass_shift_x), Some(mass_shift_y), Some(mass_shift_z)) =
                (v.mass_shift_x, v.mass_shift_y, v.mass_shift_z)
            {
                Some(Position::new(mass_shift_x, mass_shift_y, mass_shift_z))
            } else {
                None
            };
            let bound_x = Bound::new(v.bound_x1, v.bound_x2)?;
            let load = Rc::new(LoadMass::new(
                v.mass.ok_or("LoadCargo error: no mass!".to_string())?,
                bound_x,
                mass_shift,
                LoadingType::from(v.general_category),
            )?);
            log::trace!("\t Mass load_variable from cargoes:{:?} ", load);
            load_variable.push(load.clone());

            if v.is_on_deck {                
                let bound_y = if let (Some(bound_y1), Some(bound_y2)) = (v.bound_y1, v.bound_y2) {
                    Bound::new(bound_y1, bound_y2)?
                } else {
                    Bound::Full
                };
                let bound_z = if let (Some(bound_z1), Some(bound_z2)) = (v.bound_z1, v.bound_z2) {
                    Bound::new(bound_z1, bound_z2)?
                } else {
                    Bound::Full
                };
                let (vertical_area, horizontal_area) = 
                    if let (Some(vertical_area), Some(horizontal_area)) = (v.vertical_area, v.horizontal_area) {
                        (vertical_area, horizontal_area)
                } else {
                    if let (Some(bound_x), Some(bound_y), Some(bound_z)) = (bound_x.length(), bound_y.length(), bound_z.length()) {
                        (bound_z*bound_x, bound_y*bound_x)
                    } else {
                        return Err(Error::FromString(format!("Loads create error: no areas for cargo {}", v.name)));
                    }
                };
                let mass_shift = if let Some(mass_shift) = mass_shift {
                    mass_shift
                } else {
                    if let (Bound::Value(_, _), Bound::Value(_, _)) = (bound_y, bound_z) {
                        Some(Position::new(
                            bound_x.center().unwrap(),
                            bound_y.center().unwrap(),
                            bound_z.center().unwrap(),
                        ))
                    } else {
                        None
                    }
                    .ok_or(Error::FromString("Load create Desk error: no center of mass!".to_string()))?
                };
                let vertical_shift = if let (
                    Some(vertical_area_shift_x),
                    Some(vertical_area_shift_y),
                    Some(vertical_area_shift_z),
                ) = (
                    v.vertical_area_shift_x,
                    v.vertical_area_shift_y,
                    v.vertical_area_shift_z,
                ) {
                    Position::new(
                        vertical_area_shift_x,
                        vertical_area_shift_y,
                        vertical_area_shift_z,
                    )
                } else {
                    mass_shift
                };
                let desk: Rc<dyn IDesk> = Rc::new(Desk::new(
                    v.mass.ok_or("LoadCargo error: no mass!".to_string())?,
                    mass_shift,
                    bound_x,
                    bound_y,
                    bound_z,
                    vertical_area,
                    vertical_shift,
                    horizontal_area,
                    v.timber,
                    v.container.unwrap_or(false),
                ));
                desks.push(desk);
            }
            if v.timber {
                load_timber.push(load);
            }
        }

        for v in self.compartments.iter() {
            let mass_shift = if v.mass_shift_x.is_some() {
                Some(Position::new(
                    v.mass_shift_x
                        .ok_or("CompartmentData error: no mass_shift_x!".to_string())?,
                    v.mass_shift_y
                        .ok_or("CompartmentData error: no mass_shift_y!".to_string())?,
                    v.mass_shift_z
                        .ok_or("CompartmentData error: no mass_shift_z!".to_string())?,
                ))
            } else {
                None
            };
            let bound_x = Bound::new(v.bound_x1, v.bound_x2)?;
            if let Some(mass) = v.mass {
                let load = Rc::new(LoadMass::new(
                    mass,
                    bound_x,
                    mass_shift,
                    LoadingType::from(v.general_category),
                )?);
                log::trace!("\t Mass load_variable from compartments src:{:?} trg:{:?}", v, load, );
                load_variable.push(load);
            }
            if v.matter_type == CargoType::Liquid && v.m_f_s_x.is_some() && v.m_f_s_y.is_some() {
                let tank = Tank::new(
                    v.density.unwrap_or(0.),
                    v.volume.unwrap_or(0.),
                    bound_x,
                    mass_shift,
                    InertiaMoment::new(
                        v.m_f_s_x.ok_or("CompartmentData error: no x in InertiaMoment for PhysicalType::Liquid!".to_string())?,
                        v.m_f_s_y.ok_or("CompartmentData error: no y in InertiaMoment for PhysicalType::Liquid!".to_string())?,
                    ),
                    LoadingType::from(v.general_category),
                )?;
                log::trace!("\t Mass tanks from compartments:{:?} ", tank);
                let tank: Rc<dyn ITank> = Rc::new(tank);
                tanks.push(tank);
            }
            if v.matter_type == CargoType::Bulk {
                let bulk: Rc<dyn IBulk> = Rc::new(Bulk::new(
                    1. / v.density.ok_or("CompartmentData error: no density for PhysicalType::Bulk!".to_string())?,
                    v.grain_moment.ok_or("CompartmentData error: no grain_moment for PhysicalType::Bulk!".to_string())?,
                )?);
                bulks.push(bulk);
            }
        }
        *self.loads_const.borrow_mut() = Some(Rc::new(loads_const));
        *self.desks.borrow_mut() = Some(Rc::new(desks));
        *self.load_variable.borrow_mut() = Some(Rc::new(load_variable));
        *self.load_timber.borrow_mut() = Some(Rc::new(load_timber));
        *self.bulks.borrow_mut() = Some(Rc::new(bulks));
        *self.tanks.borrow_mut() = Some(Rc::new(tanks));
        Ok(())
    }
    /// Цистерны
    pub fn tanks(&self) -> Result<Rc<Vec<Rc<dyn ITank>>>, Error> {
        if self.tanks.borrow().is_none() {
            self.create()?;
        }
        Ok(Rc::clone(
            self.tanks
                .borrow()
                .as_ref()
                .ok_or("Loads tanks error: no data!".to_string())?,
        ))
    }
    /// Палубный груз
    pub fn desks(&self) -> Result<Rc<Vec<Rc<dyn IDesk>>>, Error> {
        if self.desks.borrow().is_none() {
            self.create()?;
        }
        Ok(Rc::clone(
            self.desks
                .borrow()
                .as_ref()
                .ok_or("Loads desks error: no data!".to_string())?,
        ))
    }
    /// Сыпучий груз
    pub fn bulks(&self) -> Result<Rc<Vec<Rc<dyn IBulk>>>, Error> {
        if self.bulks.borrow().is_none() {
            self.create()?;
        }
        Ok(Rc::clone(
            self.bulks
                .borrow()
                .as_ref()
                .ok_or("Loads bulks error: no data!".to_string())?,
        ))
    }
    /// Груз с переменной массой
    pub fn load_variable(&self) -> Result<Rc<Vec<Rc<LoadMass>>>, Error> {
        if self.load_variable.borrow().is_none() {
            self.create()?;
        }
        Ok(Rc::clone(
            self.load_variable
                .borrow()
                .as_ref()
                .ok_or("Loads load_variable error: no data!".to_string())?,
        ))
    }
    /// Лес
    pub fn load_timber(&self) -> Result<Rc<Vec<Rc<LoadMass>>>, Error> {
        if self.load_timber.borrow().is_none() {
            self.create()?;
        }
        Ok(Rc::clone(
            self.load_timber
                .borrow()
                .as_ref()
                .ok_or("Loads load_timber error: no data!".to_string())?,
        ))
    }
    /// Постоянная масса судна
    pub fn loads_const(&self) -> Result<Rc<Vec<Rc<LoadMass>>>, Error> {
        if self.loads_const.borrow().is_none() {
            self.create()?;
        }
        Ok(Rc::clone(
            self.loads_const
                .borrow()
                .as_ref()
                .ok_or("Loads loads_const error: no data!".to_string())?,
        ))
    }
    /// Смещение центра постоянной массы судна
    pub fn shift_const(&self) -> Position {
        self.shift_const
    }
    /// Признак палубного груза: контейнер
    pub fn have_container(&self) -> bool {
        self.cargoes.iter().any(|v| v.container.unwrap_or(false))
    }
}
