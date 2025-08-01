///
/// Результаты расчета массива скоростей хода, при которых 
/// [кажущаяся частота волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений) 
/// находится в диапазоне [основного резонанса бортовой качки](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
#[derive(Debug, Clone)]
pub struct MainResonantZoneSpeedFilterCtx {
    /// Массив скоростей хода, при которых 
    /// [кажущаяся частота волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений) 
    /// находится в диапазоне [основного резонанса бортовой качки](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
    pub main_resonant_zone_speed_filter: Vec<(f64, f64)>, // (angle, speed)
}
