use crate::algorithm::entities::icing_stab::IcingStabType;
///
/// Учет обледенения судна.
#[derive(Debug, Clone)]
pub struct IcingCtx {
    /// Тип обледенения
    pub icing_stab: IcingStabType,
    /// Масса льда на квадратный метр площади горизонтальной поверхности
    /// палубного лесного груза
    pub icing_m_timber: f64,
    /// Масса льда на квадратный метр площади парусности
    /// при учете полного обледенения
    pub icing_m_v_full: f64,
    /// Масса льда на квадратный метр площади парусности  
    /// при учете частичного обледенения
    pub icing_m_v_half: f64,
    /// Масса льда на квадратный метр площади горизонтальной
    /// поверхности при учете полного обледенения
    pub icing_m_h_full: f64,
    /// Масса льда на квадратный метр площади горизонтальной  
    /// поверхности при учете частичного обледенения
    pub icing_m_h_half: f64,
    /// Коэффициент площади парусности несплощной
    /// поверхности при учете полного обледенения
    pub icing_coef_v_area_full: f64,
    /// Коэффициент площади парусности несплощной
    /// поверхности при учете частичного обледенения
    pub icing_coef_v_area_half: f64,
    /// Коэффициент площади парусности несплощной
    /// поверхности при отсутствии обледенения
    pub icing_coef_v_area_zero: f64,
    /// Коэффициент увеличения статического момента
    /// площади парусности несплощной поверхности
    /// при учете полного обледенения
    pub icing_coef_v_moment_full: f64,
    /// Коэффициент увеличения статического момента
    /// площади парусности несплощной поверхности
    /// при учете частичного обледенения
    pub icing_coef_v_moment_half: f64,
    /// Коэффициент увеличения статического момента
    /// площади парусности несплощной поверхности
    /// при отсутствии обледенения
    pub icing_coef_v_moment_zero: f64,
}
//
impl IcingCtx {
    /// Масса льда на метр площади поверхности открытой палубы
    fn mass_desc_h(&self) -> f64 {
        match self.icing_stab {
            IcingStabType::Full => self.icing_m_h_full,
            IcingStabType::Half => self.icing_m_h_half,
            _ => 0.,
        }
    }
    /// Масса льда на метр площади палубного груза - леса
    fn mass_timber_h(&self) -> f64 {
        match self.icing_stab {
            IcingStabType::Full | IcingStabType::Half => self.icing_m_timber,
            _ => 0.,
        }
    }
    /// Масса льда на метр площади парусности
    fn mass_v(&self) -> f64 {
        match self.icing_stab {
            IcingStabType::Full => self.icing_m_v_full,
            IcingStabType::Half => self.icing_m_v_half,
            _ => 0.,
        }
    }
    /// Коэффициент увеличения площади парусности несплощной
    /// поверхности с учетом обледенения
    fn coef_v_area(&self) -> f64 {
        match self.icing_stab {
            IcingStabType::Full => self.icing_coef_v_area_full,
            IcingStabType::Half => self.icing_coef_v_area_half,
            _ => self.icing_coef_v_area_zero,
        }
    }
    /// Коэффициент увеличения площади парусности несплощной
    /// поверхности без учета обледенения
    fn coef_v_ds_area(&self) -> f64 {
        self.icing_coef_v_area_zero
    }
    /// Коэффициент увеличения статического момента
    /// площади парусности несплощной поверхности
    fn coef_v_moment(&self) -> f64 {
        match self.icing_stab {
            IcingStabType::Full => self.icing_coef_v_moment_full,
            IcingStabType::Half => self.icing_coef_v_moment_half,
            _ => self.icing_coef_v_moment_zero,
        }
    }
    /// Признал наличия обледенения
    fn is_some(&self) -> bool {
        matches!(self.icing_stab, IcingStabType::Full | IcingStabType::Half)
    }
}