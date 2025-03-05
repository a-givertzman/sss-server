//! Зависимость положения точки от некоторого значения
use super::position::Position;
/// Зависимость положения точки от некоторого значения.
/// Интерполирует значение по ключу.
pub type PosShift = super::Curve2D<Position>;
