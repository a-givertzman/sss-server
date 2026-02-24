///
/// Результаты расчета [массива скоростей движения, при которых происходит явление последовательных ударов высоких волн](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
#[derive(Debug, Clone)]
pub struct ImpactsHighWavesCtx {
    /// [Массив скоростей движения, при которых происходит явление последовательных ударов высоких волн](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
    pub impacts_high_waves: Vec<(f64, f64)> // (angle, speed)
}
