use parry3d_f64::math::Point;
///
/// Интерполяция фрейма до одинакового количества точек
/// - `points` - набор точек для интерполяции
/// - `n` - кол-во точек интерполяции
pub fn resample_line(
    points: &[Point<f64>],
    n: usize,
) -> Vec<Point<f64>> {
    if points.len() < 2 || n <= points.len() {
        return points.to_vec();
    }
    let m = points.len();
    let segs = m - 1;
    let mut lengths = Vec::with_capacity(segs);
    let mut total_len = 0.0;
    for i in 0..segs {
        let a = points[i];
        let b = points[i + 1];
        let d = ((b.x - a.x).powi(2)
            + (b.y - a.y).powi(2)
            + (b.z - a.z).powi(2))
            .sqrt();

        lengths.push(d);
        total_len += d;
    }
    let extra = n - m;
    let mut extras_per_seg = vec![0usize; segs];
    for i in 0..segs {
        extras_per_seg[i] =
            ((lengths[i] / total_len) * extra as f64).round() as usize;
    }
    let mut sum: usize = extras_per_seg.iter().sum();
    while sum > extra {
        for e in &mut extras_per_seg {
            if *e > 0 {
                *e -= 1;
                sum -= 1;
                if sum == extra { break; }
            }
        }
    }
    while sum < extra {
        for e in &mut extras_per_seg {
            *e += 1;
            sum += 1;
            if sum == extra { break; }
        }
    }
    let mut result = Vec::with_capacity(n);
    for i in 0..segs {
        let a = points[i];
        let b = points[i + 1];
        result.push(a);
        let k = extras_per_seg[i];
        for j in 1..=k {
            let t = j as f64 / (k + 1) as f64;
            let p = Point::new(
                a.x + (b.x - a.x) * t,
                a.y + (b.y - a.y) * t,
                a.z + (b.z - a.z) * t,
            );
            result.push(p);
        }
    }
    result.push(*points.last().unwrap());
    result
}