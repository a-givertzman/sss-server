//! Результаты расчета диаграммы плеч статической и динамической остойчивости

use super::lever_diagram_eval::MAX_ANGLE_CALC;
#[derive(Debug, Clone)]
pub struct LeverDiagramCtx {
    /// Результат расчета - диаграмма плеч статической остойчивости
    pub dso: Vec<(f64, f64)>,
    /// Результат расчета - диаграмма плеч статической остойчивости
    pub dso_curve: Curve<f64>,
    /// Результат расчета - диаграмма плеч динамической остойчивости
    pub ddo: Vec<(f64, f64)>,
    /// Результат расчета - диаграммы остойчивости, зависимость от угла, градусы
    pub diagram: Vec<(f64, f64, f64)>,
    /// Угол максимума диаграммы плеч статической остойчивости
    pub theta_max: f64,
    /// Углы максимумов диаграммы плеч статической остойчивости
    pub max_angles: Vec<(f64, f64)>, 
}
//
impl LeverDiagramCtx {
    /// Углы крена судна соответствующие плечу кренящего момента (angle >= 0. && angle <= elf.MAX_ANGLE_CALC.)
    fn angle(&self, lever_moment: f64) -> Result<Vec<f64>, Error> {
        let curve = self.dso_curve.as_ref().ok_or(Error::FromString(
            "LeverDiagram angle error: no dso_curve!".to_string(),
        ))?;
        let max_angle = self.theta_max.ok_or(Error::FromString(
            "LeverDiagram angle error: no max_angle!".to_string(),
        ))?;
        let curve_value = curve
            .value(max_angle)
            .map_err(|e| format!("LeverDiagram angle curve_value error: {e}"))?;
        if curve_value < lever_moment {
            let error = Error::FromString(format!(
                "LeverDiagram angle error: curve.value(max_angle:{max_angle}):{curve_value} < lever_moment:{lever_moment}!"
            ));
            log::error!("{error}");
            return Err(error);
        }
        let mut delta_angle = MAX_ANGLE_CALC;
        let mut angles = vec![
            (max_angle - delta_angle).min(MAX_ANGLE_CALC).max(-MAX_ANGLE_CALC),
            (max_angle + delta_angle).min(MAX_ANGLE_CALC).max(-MAX_ANGLE_CALC),
        ];
        for _i in 0..30 {
            let last_delta_value = lever_moment
                - curve.value(angles[0]).map_err(|e| {
                    Error::FromString(format!("LeverDiagram angle last_delta_value1 error: {e}"))
                })?;
            //    log::trace!("{}", format!("LeverDiagram angle: target:{lever_moment} angle1:{} last_delta_value:{last_delta_value} i:{_i} delta_angle:{delta_angle} ", angles[0]));
            if last_delta_value.abs() > 0.00001 {
                angles[0] =
                    (angles[0] + delta_angle * last_delta_value.signum()).min(MAX_ANGLE_CALC).max(-MAX_ANGLE_CALC);
            }
            let last_delta_value = lever_moment
                - curve.value(angles[1]).map_err(|e| {
                    Error::FromString(format!("LeverDiagram angle last_delta_value2 error: {e}"))
                })?;
            //    log::trace!("{}", format!("LeverDiagram angle: target:{lever_moment} angle2:{} last_delta_value:{last_delta_value} i:{_i} delta_angle:{delta_angle} ", angles[1]));
            if last_delta_value.abs() > 0.00001 {
                angles[1] =
                    (angles[1] - delta_angle * last_delta_value.signum()).min(MAX_ANGLE_CALC).max(-MAX_ANGLE_CALC);
            }
            delta_angle *= 0.5;
            if delta_angle < 0.0001 {
                break;
            }
        }
        Ok(angles)
    }
    /// Плечо кренящего момента соответствующие углу крена судна
    fn lever_moment(&self, angle: f64) -> Result<f64, Error> {
        if !(-MAX_ANGLE_CALC..=MAX_ANGLE_CALC).contains(&angle) {
            let error = Error::FromString(format!(
                "ILeverDiagram lever_moment error: angle {angle} < 0. || angle {angle} > max_angle"
            ));
            log::error!("{error}");
            return Err(error);
        }
        self.dso_curve
            .as_ref()
            .ok_or(Error::FromString(
                "LeverDiagram lever_moment error: no dso_curve!".to_string(),
            ))?
            .value(angle)
    }
    /// Площадь под положительной частью диаграммы статической остойчивости (rad^2)
    fn dso_area(&self, angle1: f64, angle2: f64) -> Result<f64, Error> {
        if angle1 > angle2 {
            let error = Error::FromString(format!(
                "ILeverDiagram dso_area error: angle1 {angle1} > angle2 {angle2}"
            ));
            log::error!("{error}");
            return Err(error);
        }
        Ok(self
            .dso_curve
            .as_ref()
            .ok_or(Error::FromString(
                "LeverDiagram dso_area error: no dso_curve!".to_string(),
            ))?
            .integral(angle1, angle2)?
            * PI
            / 180.)
    }
    /// Максимальное плечо диаграммы статической остойчивости в диапазонеб (м)
    fn dso_lever_max(&self, angle1: f64, angle2: f64) -> Result<f64, Error> {
        if angle1 > angle2 {
            let error = Error::FromString(format!(
                "ILeverDiagram dso_lever_max error: angle1 {angle1} > angle2 {angle2}"
            ));
            log::error!("{error}");
            return Err(error);
        }
        let dso = self.dso.as_ref().ok_or(Error::FromString(
            "ILeverDiagram dso_lever_max error: no dso!".to_string(),
        ))?;
        let mut segment = dso
            .iter()
            .filter(|v| v.0 >= angle1 && v.0 <= angle2)
            .collect::<Vec<_>>();
        segment.sort_by(|v1, v2| {
            v1.1.partial_cmp(&v2.1)
                .expect("ILeverDiagram dso_lever_max partial_cmp error!")
        });
        Ok(segment
            .last()
            .ok_or(Error::FromString(
                "ILeverDiagram dso_lever_max segment error: no values!".to_string(),
            ))?
            .1)
    }
    /// Диаграммы остойчивости, зависимость от угла, градусы
    fn diagram(&self) -> Result<Vec<(f64, f64, f64)>, Error> {
        self.diagram.clone().ok_or(Error::FromString(
            "ILeverDiagram diagram error: no diagram!".to_string(),
        ))
    }
    /// Углы максимумов диаграммы плеч статической остойчивости
    fn max_angles(&self) -> Result<Vec<(f64, f64)>, Error> {
        self.max_angles.clone().ok_or(Error::FromString(
            "ILeverDiagram max_angles error: no max_angles!".to_string(),
        ))
    }
    /// Максимальный угол для расчета диаграммы, градусы
    fn MAX_ANGLE_CALC(&self) -> f64 {
        MAX_ANGLE_CALC
    }
}