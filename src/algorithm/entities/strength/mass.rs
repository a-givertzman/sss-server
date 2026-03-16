//! Нагрузка на корпус судна
use std::{cell::RefCell, rc::Rc};

use crate::{algorithm::entities::*, kernel::error::error::Error};
use super::*;

/// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
#[derive(Clone)]
pub struct Mass {
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
//
impl Mass {
    /// Аргументы конструктора:  
    /// * loads_const - постоянная масса судна распределенная по шпациям
    /// * icing_mass - Учет обледенения судна
    /// * wetting_mass - Учет намокания палубного груза - леса
    /// * loads_variable - грузы судна
    /// * bounds - вектор разбиения на отрезки для эпюров
    /// * results, parameters - Набор результатов расчетов для записи в БД
    pub fn new(
        loads_const: Rc<Vec<Rc<LoadMass>>>,
        icing_mass: Rc<dyn IIcingMass>,
        wetting_mass: Rc<dyn IWettingMass>,
        loads_variable: Rc<Vec<Rc<LoadMass>>>,
        bounds: Rc<Bounds>,
        results: Rc<dyn IResults>,
        parameters: Rc<dyn IParameters>,
    ) -> Self {
        Self {
            loads_const,
            icing_mass,
            wetting_mass,
            loads_variable,
            bounds,
            results,
            parameters,
            ballast: Rc::new(RefCell::new(None)),
            stores: Rc::new(RefCell::new(None)),
            icing: Rc::new(RefCell::new(None)),
            wetting: Rc::new(RefCell::new(None)),
            cargo: Rc::new(RefCell::new(None)),
            bulkhead: Rc::new(RefCell::new(None)),
            lightship: Rc::new(RefCell::new(None)),
            sum: Rc::new(RefCell::new(None)),
            mass_values: Rc::new(RefCell::new(None)),
        }
    }
    /// Ленивое вычисление
    fn calculate(&self) -> Result<(), Error> {
        *self.ballast.borrow_mut() = Some(self.ballast()?);
        *self.stores.borrow_mut() = Some(self.stores()?);
        *self.icing.borrow_mut() = Some(self.icing()?);
        *self.wetting.borrow_mut() = Some(self.wetting()?);
        *self.cargo.borrow_mut() = Some(self.cargo()?);
        *self.bulkhead.borrow_mut() = Some(self.bulkhead()?);
        *self.lightship.borrow_mut() = Some(self.lightship()?);
        *self.sum.borrow_mut() = Some(self.sum()?);
        *self.mass_values.borrow_mut() = Some(self.values()?);
        Ok(())
    }
    /// Суммарная масса балласта
    fn ballast(&self) -> Result<f64, Error> {
        let mut ballast = 0.;
        for v in self
            .loads_variable
            .iter()
            .filter(|v| v.load_type() == LoadingType::Ballast)
        {
            ballast += v.value(&Bound::Full)?;
        }
        Ok(ballast)
    }
    /// Суммарная масса запасов
    fn stores(&self) -> Result<f64, Error> {
        let mut stores = 0.;
        for v in self
            .loads_variable
            .iter()
            .filter(|v| v.load_type() == LoadingType::Stores)
        {
            stores += v.value(&Bound::Full)?;
        }
        Ok(stores)
    }
    /// Суммарная масса обледенения
    fn icing(&self) -> Result<f64, Error> {
        self.icing_mass.mass(&Bound::Full)
    }
    /// Суммарная масса намокания
    fn wetting(&self) -> Result<f64, Error> {
        self.wetting_mass.mass(&Bound::Full)
    }
    /// Суммарная масса груза
    fn cargo(&self) -> Result<f64, Error> {
        let mut cargo = 0.;
        for v in self
            .loads_variable
            .iter()
            .filter(|v| v.load_type() == LoadingType::Cargo)
        {
            cargo += v.value(&Bound::Full)?;
        }
        Ok(cargo)
    }
    /// Суммарная масса зерновых перегородок
    fn bulkhead(&self) -> Result<f64, Error> {
        let mut bulkhead = 0.;
        for v in self
            .loads_variable
            .iter()
            .filter(|v| v.load_type() == LoadingType::Bulkhead)
        {
            bulkhead += v.value(&Bound::Full)?;
        }
        Ok(bulkhead)
    }
    /// Суммарная масса корпуса
    fn lightship(&self) -> Result<f64, Error> {
        let mut lightship = 0.;
        for v in self.loads_const.iter() {
            lightship += v.value(&Bound::Full)?;
        }
        Ok(lightship)
    }
    /// Суммарная масса корпуса и грузов судна
    fn sum(&self) -> Result<f64, Error> {
        let ballast = self.ballast()?;
        let stores = self.stores()?;
        let cargo = self.cargo()?;
        let bulkhead = self.bulkhead()?;
        let deadweight = ballast + stores + cargo + bulkhead; // Суммарная масса переменного груза
        let lightship = self.lightship()?;
        let icing = self.icing()?;
        let wetting = self.wetting()?;
        let mass_sum = deadweight + lightship + wetting + icing;
        self.parameters.add(ParameterID::Displacement, mass_sum);
        self.parameters.add(ParameterID::MassBallast, ballast);
        self.parameters.add(ParameterID::MassStores, stores);
        self.parameters.add(ParameterID::MassBulkhead, bulkhead);
        self.parameters.add(ParameterID::MassCargo, cargo);
        self.parameters.add(ParameterID::MassDeadweight, deadweight);
        self.parameters.add(ParameterID::MassLightship, lightship);
        self.parameters.add(ParameterID::MassIcing, icing);
        self.parameters.add(ParameterID::MassWetting, wetting);
        log::info!(
            "\t Mass ballast:{ballast}, stores:{stores}, bulkhead:{bulkhead}
            cargo:{cargo}, deadweight:{deadweight}, lightship:{lightship},
            icing:{icing}, wetting:{wetting} sum:{mass_sum}"
        );
        Ok(mass_sum)
    }
    /// Распределение массы по вектору разбиения
    fn values(&self) -> Result<Vec<f64>, Error> {
        let mut vec_hull = Vec::new();
        let mut vec_equipment = Vec::new();
        let mut vec_bulkhead = Vec::new();
        let mut vec_ballast = Vec::new();
        let mut vec_store = Vec::new();
        let mut vec_cargo = Vec::new();
        let mut vec_icing = Vec::new();
        let mut vec_wetting = Vec::new();
        let mut vec_sum = Vec::new();
        let mut res: Vec<f64> = Vec::new();
        for b in self.bounds.iter() {
            let mut hull = 0.;
            for v in self
                .loads_const
                .iter()
                .filter(|v| v.load_type() == LoadingType::Hull)
            {
                hull += v.value(b)?;
            }
            vec_hull.push(hull);
            let mut equipment = 0.;
            for v in self
                .loads_const
                .iter()
                .filter(|v| v.load_type() == LoadingType::Equipment)
            {
                equipment += v.value(b)?;
            }
            vec_equipment.push(equipment);
            let mut bulkhead = 0.;
            for v in self
                .loads_variable
                .iter()
                .filter(|v| v.load_type() == LoadingType::Bulkhead)
            {
                bulkhead += v.value(b)?;
            }
            vec_bulkhead.push(bulkhead);
            let mut ballast = 0.;
            for v in self
                .loads_variable
                .iter()
                .filter(|v| v.load_type() == LoadingType::Ballast)
            {
                ballast += v.value(b)?;
            }
            vec_ballast.push(ballast); 
            let mut store = 0.;
            for v in self
                .loads_variable
                .iter()
                .filter(|v| v.load_type() == LoadingType::Stores)
            {
                store += v.value(b)?;
            }
            vec_store.push(store);
            let mut cargo = 0.;
            for v in self
                .loads_variable
                .iter()
                .filter(|v| v.load_type() == LoadingType::Cargo)
            {
                cargo += v.value(b)?;
            }
            vec_cargo.push(cargo);
            let icing = self.icing_mass.mass(b)?;
            vec_icing.push(icing);
            let wetting = self.wetting_mass.mass(b)?;
            vec_wetting.push(wetting);
            res.push(hull + equipment + bulkhead + ballast + store + cargo + icing + wetting);
        }
        vec_hull.push(vec_hull.iter().sum());
        vec_equipment.push(vec_equipment.iter().sum());
        vec_bulkhead.push(vec_bulkhead.iter().sum());
        vec_ballast.push(vec_ballast.iter().sum());
        vec_store.push(vec_store.iter().sum());
        vec_cargo.push(vec_cargo.iter().sum());
        vec_icing.push(vec_icing.iter().sum());
        vec_wetting.push(vec_wetting.iter().sum());
        vec_sum.append(&mut res.clone());
        vec_sum.push(res.iter().sum());
        log::info!("\t Mass values:{:?} ", res);
        self.results.add("value_mass_hull".to_owned(), vec_hull);
        self.results
            .add("value_mass_equipment".to_owned(), vec_equipment);
        self.results
            .add("value_mass_bulkhead".to_owned(), vec_bulkhead);
        self.results
            .add("value_mass_ballast".to_owned(), vec_ballast);
        self.results.add("value_mass_store".to_owned(), vec_store);
        self.results.add("value_mass_cargo".to_owned(), vec_cargo);
        self.results.add("value_mass_icing".to_owned(), vec_icing);
        self.results
            .add("value_mass_wetting".to_owned(), vec_wetting);
        self.results.add("value_mass_sum".to_owned(), vec_sum);
        Ok(res)
    }
}
/// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
impl IMass for Mass {
    /// Суммарная масса балласта
    fn ballast(&self) -> Result<f64, Error> {
        if self.ballast.borrow().is_none() {
            self.calculate()?;
        }
        self.ballast
            .borrow()
            .ok_or(Error::FromString("Mass ballast error: no value".to_owned()))
    }
    /// Суммарная масса запасов
    fn stores(&self) -> Result<f64, Error> {
        if self.stores.borrow().is_none() {
            self.calculate()?;
        }
        self.stores
            .borrow()
            .ok_or(Error::FromString("Mass stores error: no value".to_owned()))
    }
    /// Суммарная масса обледенения
    fn icing(&self) -> Result<f64, Error> {
        if self.icing.borrow().is_none() {
            self.calculate()?;
        }
        self.icing
            .borrow()
            .ok_or(Error::FromString("Mass icing error: no value".to_owned()))
    }
    /// Суммарная масса намокания
    fn wetting(&self) -> Result<f64, Error> {
        if self.wetting.borrow().is_none() {
            self.calculate()?;
        }
        self.wetting
            .borrow()
            .ok_or(Error::FromString("Mass wetting error: no value".to_owned()))
    }
    /// Суммарная масса груза
    fn cargo(&self) -> Result<f64, Error> {
        if self.cargo.borrow().is_none() {
            self.calculate()?;
        }
        self.cargo
            .borrow()
            .ok_or(Error::FromString("Mass cargo error: no value".to_owned()))
    }
    /// Суммарная масса зерновых перегородок
    fn bulkhead(&self) -> Result<f64, Error> {
        if self.bulkhead.borrow().is_none() {
            self.calculate()?;
        }
        self.bulkhead.borrow().ok_or(Error::FromString(
            "Mass bulkhead error: no value".to_owned(),
        ))
    }
    /// Суммарная масса корпуса
    fn lightship(&self) -> Result<f64, Error> {
        if self.lightship.borrow().is_none() {
            self.calculate()?;
        }
        self.lightship.borrow().ok_or(Error::FromString(
            "Mass lightship error: no value".to_owned(),
        ))
    }
    /// Суммарная масса корпуса и грузов судна
    fn sum(&self) -> Result<f64, Error> {
        if self.sum.borrow().is_none() {
            self.calculate()?;
        }
        self.sum
            .borrow()
            .ok_or(Error::FromString("Mass sum error: no value".to_owned()))
    }
    /// Распределение массы по вектору разбиения
    fn values(&self) -> Result<Vec<f64>, Error> {
        if self.mass_values.borrow().is_none() {
            self.calculate()?;
        }
        self.mass_values
            .borrow()
            .clone()
            .ok_or(Error::FromString("Mass values error: no values".to_owned()))
    }
}

#[doc(hidden)]
pub trait IMass {
    /// Суммарная масса балласта
    fn ballast(&self) -> Result<f64, Error>;
    /// Суммарная масса запасов
    fn stores(&self) -> Result<f64, Error>;
    /// Суммарная масса обледенения
    fn icing(&self) -> Result<f64, Error>;
    /// Суммарная масса намокания
    fn wetting(&self) -> Result<f64, Error>;
    /// Суммарная масса груза
    fn cargo(&self) -> Result<f64, Error>;
    /// Суммарная масса зерновых перегородок
    fn bulkhead(&self) -> Result<f64, Error>;
    /// Суммарная масса корпуса
    fn lightship(&self) -> Result<f64, Error>;
    /// Суммарная масса
    fn sum(&self) -> Result<f64, Error>;
    /// Распределение массы по вектору разбиения
    fn values(&self) -> Result<Vec<f64>, Error>;
}
// заглушка для тестирования
#[doc(hidden)]
pub struct FakeMass {
    ballast: f64,
    stores: f64,
    icing: f64,
    wetting: f64,
    cargo: f64,
    bulkhead: f64,
    lightship: f64,
    sum: f64,
    values: Vec<f64>,
}
#[doc(hidden)]
#[allow(dead_code)]
impl FakeMass {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ballast: f64,
        stores: f64,
        icing: f64,
        wetting: f64,
        cargo: f64,
        bulkhead: f64,
        lightship: f64,
        sum: f64,
        values: Vec<f64>,
    ) -> Self {
        Self {
            ballast,
            stores,
            icing,
            wetting,
            cargo,
            bulkhead,
            lightship,
            sum,
            values,
        }
    }
}
#[doc(hidden)]
impl IMass for FakeMass {
    fn ballast(&self) -> Result<f64, Error> {
        Ok(self.ballast)
    }
    fn stores(&self) -> Result<f64, Error> {
        Ok(self.stores)
    }
    fn icing(&self) -> Result<f64, Error> {
        Ok(self.icing)
    }
    fn wetting(&self) -> Result<f64, Error> {
        Ok(self.wetting)
    }
    fn cargo(&self) -> Result<f64, Error> {
        Ok(self.cargo)
    }
    fn bulkhead(&self) -> Result<f64, Error> {
        Ok(self.bulkhead)
    }
    fn lightship(&self) -> Result<f64, Error> {
        Ok(self.lightship)
    }
    fn sum(&self) -> Result<f64, Error> {
        Ok(self.sum)
    }
    fn values(&self) -> Result<Vec<f64>, Error> {
        Ok(self.values.clone())
    }
}
