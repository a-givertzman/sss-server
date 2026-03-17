use crate::algorithm::eval::seakeeping::entities::graham::graham::GrahamScan;

pub struct Clusterization {
    contour: Vec<(f64, f64)>,
    alpha: f64,
}

impl Clusterization {
    ///
    /// Новый экземпляр класса [Clusterization]
    pub fn new(contour: Vec<(f64, f64)>, alpha: f64) -> Self {
        let mut cartes_contour: Vec<(f64, f64)> = Vec::new();
        for point in contour.clone() {
            let cartes_point = GrahamScan::polar_to_cartesian(point.0, point.1);
            cartes_contour.push(cartes_point);
        }
        cartes_contour.sort_by(
            |a, b| 
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        );
        Self { 
            contour: cartes_contour,
            alpha,
        }
    }
    ///
    /// Разделение кластеров
    pub fn eval(&mut self) -> Option<Vec<Vec<(f64, f64)>>> {
        let step_x: f64 = 0.1;
        let step_y: f64 = 0.1;
        let delta_min = (step_x.powi(2) + step_y.powi(2)).sqrt();
        let a = (step_x.powi(2) + (2.0 * step_y).powi(2)).sqrt();
        let b = ((2.0 * step_x).powi(2) + step_y.powi(2)).sqrt();
        let delta_max = (a + b) / 2.0;
        let delta = (delta_min + delta_max) / 2.0;
        match self.find_gap(self.find_points_on_line(1e-10), delta) {
            Some((start, end)) => {
                let mid_point = (
                    (start.0 + end.0) / 2.0,
                    (start.1 + end.1) / 2.0
                );
                let line_angle = self.alpha.to_radians();
                let perpendicular_angle = line_angle + std::f64::consts::FRAC_PI_2;
                let segment_length = 10.0;
                let points_on_line = Self::generate_points_on_line(mid_point, perpendicular_angle, segment_length, 10);
                let (left_cluster, right_cluster) = self.split_clusters_by_line(&points_on_line);
                let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                if !left_cluster.is_empty() {
                    let mut cartes_contour: Vec<(f64, f64)> = Vec::new();
                    for point in left_cluster.clone() {
                        let cartes_point = GrahamScan::cartesian_to_polar(point);
                        cartes_contour.push(cartes_point);
                    }
                    cartes_contour.sort_by(
                        |a, b| 
                        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                    );
                    result.push(cartes_contour);
                }
                if !right_cluster.is_empty() {
                    let mut cartes_contour: Vec<(f64, f64)> = Vec::new();
                    for point in right_cluster.clone() {
                        let cartes_point = GrahamScan::cartesian_to_polar(point);
                        cartes_contour.push(cartes_point);
                    }
                    cartes_contour.sort_by(
                        |a, b| 
                        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                    );
                    result.push(cartes_contour);
                }
                Some(result)
            },
            None => {
                log::debug!("There is no subzones");
                let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
                let mut cartes_contour: Vec<(f64, f64)> = Vec::new();
                for point in self.contour.clone() {
                    let cartes_point = GrahamScan::cartesian_to_polar(point);
                    cartes_contour.push(cartes_point);
                }
                result.push(cartes_contour);
                Some(result)
            },
        }
    }
    ///
    /// Разделение точек на два кластера относительно линии
    fn split_clusters_by_line(&self, line_points: &[(f64, f64)]) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        if line_points.len() < 2 {
            return (self.contour.clone(), Vec::new());
        }
        // Получаем уравнение прямой по двум точкам
        let p1 = line_points[0];
        let p2 = line_points[line_points.len() - 1];
        let (a, b, c) = Self::line_equation(p1, p2);
        let mut left_cluster = Vec::new();
        let mut right_cluster = Vec::new();
        for &point in &self.contour {
            let side = Self::point_side_of_line(point, a, b, c);
            if side > 0.0 {
                right_cluster.push(point);
            } else if side < 0.0 {
                left_cluster.push(point);
            } else {
                left_cluster.push(point);
            }
        }
        (left_cluster, right_cluster)
    }
    ///
    /// Уравнение прямой: Ax + By + C = 0
    fn line_equation(p1: (f64, f64), p2: (f64, f64)) -> (f64, f64, f64) {
        let a = p2.1 - p1.1;
        let b = p1.0 - p2.0;
        let c = p2.0 * p1.1 - p1.0 * p2.1;
        (a, b, c)
    }
    ///
    /// Определение положения точки относительно прямой
    /// Возвращает >0 если точка справа, <0 если слева, =0 если на прямой
    fn point_side_of_line(point: (f64, f64), a: f64, b: f64, c: f64) -> f64 {
        a * point.0 + b * point.1 + c
    }
    ///
    /// Функция для генерации нескольких точек на прямой
    fn generate_points_on_line(mid_point: (f64, f64), angle: f64, total_length: f64, num_points: usize) -> Vec<(f64, f64)> {
        let half_length = total_length / 2.0;
        let step = total_length / (num_points - 1) as f64;
        let mut points = Vec::new();
        for i in 0..num_points {
            let distance = -half_length + (i as f64) * step;
            let x = mid_point.0 + distance * angle.cos();
            let y = mid_point.1 + distance * angle.sin();
            points.push((x, y));
        }
        points
    }
    ///
    /// нахождение разрыва между подзонами
    fn find_gap(&self, mut points_on_line: Vec<(f64,f64)>, delta: f64) -> Option<((f64,f64),(f64,f64))> {
        if points_on_line.is_empty() {
            return None;
        }
        points_on_line.sort_by(
            |a, b| 
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        );
        for i in 0..points_on_line.len() - 1 {
            let p1  = points_on_line[i];
            let p2  = points_on_line[i + 1];
            let gap = self.distance(p1, p2);
            if gap > delta {
                return Some((p1,p2))
            }
        }
        None
    } 
    ///
    ///  евклидово расстояние
    fn distance(&self, p1: (f64,f64), p2: (f64,f64)) -> f64 {
        ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()
    }
    ///
    /// Нахождение точек на прямой
    fn find_points_on_line(&self, tolerance: f64) -> Vec<(f64, f64)> {
        let cartesian_angle = self.alpha;
        let k = cartesian_angle.to_radians().tan();
        let mut points_on_line = Vec::new();
        for &point in &self.contour {
            let (x, y) = point;
            if cartesian_angle.to_radians().cos().abs() < tolerance {
                if x.abs() < tolerance {
                    points_on_line.push(point);
                }
            } else {
                let expected_y = k * x;
                if (y - expected_y).abs() < tolerance {
                    points_on_line.push(point);
                }
            }
        }
        points_on_line
    }
}