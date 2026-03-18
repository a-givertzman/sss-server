//! Результаты расчета диаграммы плеч статической и динамической остойчивости
//
use crate::algorithm::entities::math::curve::*;
use sal_core::error::Error;

// Максимальный угол для расчета диаграммы, градусы
pub(crate) const MAX_LEVER_ANGLE_CALC: f64 = 60.;

#[derive(Debug, Clone)]
pub struct LeverDiagramCtx {
    /// Результат расчета - диаграмма плеч статической остойчивости
    pub dso: Vec<(f64, f64)>,
    /// Результат расчета - диаграмма плеч статической остойчивости
    pub dso_curve: Curve<f64>,
    /// Результат расчета - диаграмма плеч динамической остойчивости
    pub ddo: Vec<(f64, f64)>,
    /// Угол максимума диаграммы плеч статической остойчивости
    pub theta_max: f64,
    /// Углы максимумов диаграммы плеч статической остойчивости
    pub max_angles: Vec<(f64, f64)>,
    ///  Угол входа в воду кромки палубы, градусы
    pub entry_angle: f64,
    ///  Угол заливания отверстий, градусы
    pub flooding_angle: f64,
}
//
impl LeverDiagramCtx {
    /// Углы крена судна соответствующие плечу кренящего момента (angle >= 0. && angle <= elf.MAX_LEVER_ANGLE_CALC.)
    pub fn angle(&self, lever_moment: f64) -> Result<Vec<f64>, Error> {
        angle(self.theta_max, &self.dso_curve, lever_moment)
    }
    /// Плечо кренящего момента соответствующие углу крена судна
    pub fn lever_moment(&self, angle: f64) -> Result<f64, Error> {
        if !(-MAX_LEVER_ANGLE_CALC..=MAX_LEVER_ANGLE_CALC).contains(&angle) {
            let error = Error::new(
                "LeverDiagram",
                format!("lever_moment error: angle {angle} < 0. || angle {angle} > max_angle"),
            );
            log::error!("{error}");
            return Err(error);
        }
        self.dso_curve.value(angle).map_err(|e| {
            Error::new(
                "LeverDiagram",
                format!("lever_moment dso_curve.value from {angle} error: {e}"),
            )
        })
    }
    /// Площадь под положительной частью диаграммы статической остойчивости (rad^2)
    pub fn dso_area(&self, angle1: f64, angle2: f64) -> Result<f64, Error> {
        if angle1 > angle2 {
            let error = Error::new(
                "LeverDiagram",
                format!("dso_area error: angle1 {angle1} > angle2 {angle2}"),
            );
            log::error!("{error}");
            return Err(error);
        }
        Ok(self.dso_curve.integral(angle1, angle2)? * std::f64::consts::PI / 180.)
    }
    /// Максимальное плечо диаграммы статической остойчивости в диапазонеб (м)
    pub fn dso_lever_max(&self, angle1: f64, angle2: f64) -> Result<f64, Error> {
        if angle1 > angle2 {
            let error = Error::new(
                "LeverDiagram",
                format!("dso_lever_max error: angle1 {angle1} > angle2 {angle2}"),
            );
            log::error!("{error}");
            return Err(error);
        }
        let mut segment = self
            .dso
            .iter()
            .filter(|v| v.0 >= angle1 && v.0 <= angle2)
            .collect::<Vec<_>>();
        segment.sort_by(|v1, v2| {
            v1.1.partial_cmp(&v2.1)
                .expect("LeverDiagram dso_lever_max partial_cmp error!")
        });
        Ok(segment
            .last()
            .ok_or(Error::new(
                "LeverDiagram",
                "dso_lever_max segment error: no values!".to_string(),
            ))?
            .1)
    }
    /// Углы максимумов диаграммы плеч статической остойчивости
    pub fn max_angles(&self) -> Vec<(f64, f64)> {
        self.max_angles.clone()
    }
}

/// Углы крена судна соответствующие плечу кренящего момента (angle >= 0. && angle <= self.max_angle_calc.)
pub(crate) fn angle(
    max_angle: f64,
    curve: &Curve<f64>,
    lever_moment: f64,
) -> Result<Vec<f64>, Error> {
    let curve_value = curve
        .value(max_angle)
        .map_err(|e| format!("angle curve_value error: {e}"))?;
    if curve_value < lever_moment {
        let error = Error::new(
            "lever_diagram",
            format!(
                "angle error: curve.value(max_angle:{max_angle}):{curve_value} < lever_moment:{lever_moment}!"
            ),
        );
        log::error!("{error}");
        return Err(error);
    }
    let angle = |start_angle: f64, mut step: f64, epsilon: f64| -> Result<f64, Error> {
        let mut angle = start_angle;
        let mut last_delta_moment: f64 = -1.;
        for _i in 0..100 {
            let delta_moment = lever_moment
                - curve.value(angle).map_err(|e| {
                Error::new(
                    "lever_diagram",
                    format!("angle last_delta_value1 error: {e}"),
                )
            })?;
            //             log::trace!("{}", format!("LeverDiagram angle: target:{lever_moment} angle:{angle} last_delta_moment:{last_delta_moment} i:{_i} delta_moment:{delta_moment}"));
            if delta_moment.abs() <= epsilon {
                break;
            }
            if delta_moment.signum() != last_delta_moment.signum() {
                step = -step / 2.;
                last_delta_moment = delta_moment;
            }
            angle = (angle + step)
                .min(MAX_LEVER_ANGLE_CALC)
                .max(-MAX_LEVER_ANGLE_CALC);
        }
        Ok(angle)
    };
    let angle1: f64 = angle(max_angle, -1., 0.00001)?;
    let angle2 = angle(max_angle, 1., 0.00001)?;
  //  log::trace!("{}", format!("LeverDiagram angle: lever_moment:{lever_moment} max_angle:{max_angle} angle1:{angle1} angle2:{angle2}"));
    Ok(vec![angle1, angle2])
}
