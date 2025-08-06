///
/// Перевод курсовых углов волнения из северо-восточной системы координат β в курсовые углы волнения 
/// относительно ДП судна α и наоборот
pub struct RecalculationCourseAngular {
}
//
//
impl RecalculationCourseAngular {
    ///
    /// Перевод в северо-восточную систему координат
    pub fn to_northeastern(course_angle: f64, array: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
        let result = array
        .iter()
        .map(|(angle, speed)| {
            if (angle + course_angle) >= 360.0 {
                (
                    angle + course_angle - 360.0 ,
                    *speed,
                )
            } else {
                (
                    angle + course_angle,
                    *speed,
                )
            }
        }).collect::<Vec<(f64, f64)>>();
        return result;
    }
    ///
    /// Перевод курсовые углы волнения относительно ДП судна
    pub fn to_course_angle(course_angle: f64, array: Vec<(f64, f64)>) -> Vec<(f64,f64)> {
        let result = array
        .iter()
        .map(|(angle, speed)| {
            if (angle - course_angle) < 0.0 {
                (
                    angle - course_angle + 360.0,
                    *speed,
                )
            } else {
                (
                    angle - course_angle,
                    *speed,
                )
            }
        }).collect::<Vec<(f64, f64)>>();
        return result;
    }
}