use std::f64::consts::PI;

///
/// [Graham scan](https://en.wikipedia.org/wiki/Graham_scan)
pub struct GrahamScan {
    contour: Vec<(f64, f64)>,
}
//
impl GrahamScan {
    ///
    /// New instance [GrahamScan]
    pub fn new(contour: Vec<(f64, f64)>) -> Self {
        Self { contour }
    }
    ///
    /// Преобразование полярных координат в декартовы
    pub fn polar_to_cartesian(angle_deg: f64, radius: f64) -> (f64, f64) {
        let angle_rad = angle_deg.to_radians();
        (
            radius * angle_rad.cos(),
            radius * angle_rad.sin()
        )
    }
    ///
    /// Преобразование декартовых координат в полярные
    pub fn cartesian_to_polar(point: (f64, f64)) -> (f64, f64) {
        let radius = (point.1.powi(2) + point.0.powi(2)).sqrt();
        if radius == 0.0 {
            return (0.0, 0.0);
        }
        let mut angle_rad = point.1.atan2(point.0);
        if angle_rad < 0.0 {
            angle_rad += 2.0 * PI;
        }
        let angle_deg = angle_rad.to_degrees();
        (angle_deg, radius)
    }
    ///
    /// Find the lowest y-coordinate and leftmost point, called P0
    fn find_p0(points: &[(f64, f64)]) -> (f64, f64) {
        let mut p0 = points[0];
        for point in points {
            // Find point with minimum y, and if equal y, minimum x
            if point.1 < p0.1 || (point.1 == p0.1 && point.0 < p0.0) {
                p0 = *point;
            }
        }
        p0
    }
    ///
    /// Polar angle between p0 and p1
    fn polar_angle(p0: (f64, f64), p1: (f64, f64)) -> f64 {
        let dx = p1.0 - p0.0;
        let dy = p1.1 - p0.1;
        dy.atan2(dx)
    }
    ///
    /// Squared distance between two points
    fn distance_sq(p1: (f64, f64), p2: (f64, f64)) -> f64 {
        let dx = p1.0 - p2.0;
        let dy = p1.1 - p2.1;
        dx * dx + dy * dy
    }
    ///
    /// Orientation of three points (cross product)
    fn orientation(p: (f64, f64), q: (f64, f64), r: (f64, f64)) -> i32 {
        let val = (q.1 - p.1) * (r.0 - q.0) - (q.0 - p.0) * (r.1 - q.1);
        if val.abs() < 1e-10 { 0 } // colinear (with floating point tolerance)
        else if val > 0.0 { 1 } // clockwise
        else { 2 } // counter-clockwise
    }
    ///
    /// Finding convex hull by Graham scan
    pub fn eval(&self) -> Vec<(f64, f64)> {
        if self.contour.len() < 3 {
            return self.contour.clone(); // Convex hull of <3 points is the set itself
        }
        let mut cartes_contour: Vec<(f64 ,f64)> = Vec::new();
        for point in self.contour.clone() {
            let cartes_point = GrahamScan::polar_to_cartesian(point.0, point.1);
            cartes_contour.push(cartes_point);
        }
        let p0 = Self::find_p0(&cartes_contour);
        // Remove p0 from the list and sort the remaining points
        let mut points: Vec<(f64, f64)> = cartes_contour
            .iter()
            .filter(|&&p| p != p0)
            .cloned()
            .collect();
        // Sort by polar angle, then by distance
        points.sort_by(|&a, &b| {
            let angle_a = Self::polar_angle(p0, a);
            let angle_b = Self::polar_angle(p0, b);
            
            angle_a.partial_cmp(&angle_b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    let dist_a = Self::distance_sq(p0, a);
                    let dist_b = Self::distance_sq(p0, b);
                    dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        // Remove collinear points (keep only the farthest one)
        let mut filtered_points = Vec::new();
        filtered_points.push(p0);
        for mut i in 0..points.len() {
            // Skip duplicates and collinear points that are closer
            while i + 1 < points.len() && 
                  Self::orientation(p0, points[i], points[i + 1]) == 0 {
                i += 1;
            }
            filtered_points.push(points[i]);
        }
        if filtered_points.len() < 3 {
            return filtered_points; // Not enough points for convex hull
        }
        // Build the convex hull using a stack
        let mut stack: Vec<(f64, f64)> = Vec::new();
        stack.push(filtered_points[0]);
        stack.push(filtered_points[1]);
        stack.push(filtered_points[2]);
        for i in 3..filtered_points.len() {
            while stack.len() >= 2 {
                let top = stack[stack.len() - 1];
                let next_to_top = stack[stack.len() - 2];
                if Self::orientation(next_to_top, top, filtered_points[i]) != 2 {
                    stack.pop();
                } else {
                    break;
                }
            }
            stack.push(filtered_points[i]);
        }
        let mut polar_result: Vec<(f64, f64)> = Vec::new();
        for point in stack {
            let polar_point = GrahamScan::cartesian_to_polar(point);
            polar_result.push(polar_point);
        }
        polar_result
    }
}