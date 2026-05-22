use sal_core::error::Error;
use serde::{Deserialize, Serialize};
use sal_3dlib_core::math::*;
/// Тип обледенения горизонтальной площади палубного груза - леса
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum IcingTimberType {
    #[serde(alias = "full")]
    Full,
    #[serde(alias = "half left")]
    HalfLeft,
    #[serde(alias = "half right")]
    HalfRight,
    #[serde(alias = "bow")]
    Bow,
}
///
/// Ограничение горизонтальной площади обледенения палубного груза - леса
impl IcingTimberType {
    pub fn from_str(src: &str) -> Result<Self, Error> {
        Ok(match src.trim().to_lowercase().as_str() {
            "full" => IcingTimberType::Full,
            "half left" => IcingTimberType::HalfLeft,
            "half right" => IcingTimberType::HalfRight,
            "bow" => IcingTimberType::Bow,
            src => return Err(Error::from(format!("IcingTimberType.from_str | from_str error: no type {src}"))),
        })
    }
}
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct IcingTimberBoundCtx {
    /// Ширина корпуса судна  
    pub width: f64,
    /// Длинна корпуса судна  
    pub length: f64,
    /// Тип обледенения  
    pub icing_timber_stab: IcingTimberType,
}
//
impl IcingTimberBoundCtx {
    /// Основной конструктор
    /// * width - Ширина корпуса судна  
    /// * length - Длинна корпуса судна    
    /// * icing_timber_stab - Тип обледенения   
    pub fn new(width: f64, length: f64, icing_timber_stab: IcingTimberType) -> Self {
        Self {
            width,
            length,
            icing_timber_stab,
        }
    }
    /// Ограничение по x
    pub fn bound_x(&self) -> Result<Bound, Error> {
        Ok(match self.icing_timber_stab {
            IcingTimberType::Bow => Bound::new(self.length / 6., self.length / 2.)?,
            _ => Bound::Full,
        })
    }
    /// Ограничение по y
    pub fn bound_y(&self) -> Result<Bound, Error> {
        Ok(match self.icing_timber_stab {
            IcingTimberType::HalfLeft => Bound::new(-self.width / 2., 0.)?,
            IcingTimberType::HalfRight => Bound::new(0., self.width / 2.)?,
            _ => Bound::Full,
        })
    }
}
