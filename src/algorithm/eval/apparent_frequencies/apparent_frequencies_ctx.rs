///
/// Результаты расчета массива [кажущихся частот волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
#[derive(Debug, Clone)]
pub struct ApparentFrequenciesCtx {
    /// Массив [кажущихся частот волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
    pub apparent_frequencies: Vec<(f64, f64)>, // (angle, speed)
}
