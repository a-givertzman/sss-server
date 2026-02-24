use std::collections::HashMap;
use crate::{
    algorithm::eval::seakeeping::{
        entities::graham::graham::GrahamScan,
        eval::{
            hitting_zones::hitting_point_ctx::HittingZonesCtx,
            impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx,
            main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx,
            move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx,
            parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx
        }
    },
    kernel::{
        Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{
        ContextRead,
        ContextReadRef,
        ContextWrite,
        InitialCtx
    }
};
///
/// Расчет пересечения с зонами резонанса
pub struct HittingZonesEval {
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl HittingZonesEval {
    ///
    /// Новый экземпляр [HittingZonesEval]
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        Self {
            ctx: Box::new(ctx),
        }
    }
    ///
    /// Проверка на попадание точки в полигон
    fn point_in_polygon(&self, point: (f64, f64), polygon: &[(f64, f64)]) -> bool {
        if polygon.len() < 3 {
            return false;
        }
        let (x, y) = point;
        let mut crossings = 0;
        for i in 0..polygon.len() {
            let j = (i + 1) % polygon.len();
            let (x1, y1) = polygon[i];
            let (x2, y2) = polygon[j];
            // Проверка на нахождение точки на вершине
            if (x == x1 && y == y1) || (x == x2 && y == y2) {
                return true;
            }
            // Проверка на нахождение точки на горизонтальном ребре
            if (y1 == y2) && (y == y1) && (x >= x1.min(x2)) && (x <= x1.max(x2)) {
                return true;
            }
            // Проверка пересечения луча, идущего вправо от точки, с ребром полигона
            if ((y1 <= y && y < y2) || (y2 <= y && y < y1)) && 
            (x < (x2 - x1) * (y - y1) / (y2 - y1) + x1) {
                crossings += 1;
            }
        }
        crossings % 2 == 1
    }
    ///
    /// Проверка на попадание в основную зону резонанса
    fn check_main(&self, main_zone: Vec<Vec<(f64,f64)>>, current_speed: f64, course_angle: f64) -> bool {
        for cluster in main_zone {
            if self.point_in_polygon((course_angle, current_speed), &cluster) {
                return true; // возвращаем true при первом попадании
            }
        }
        false
    }
    ///
    /// Проверка на попадание в параметрическую зону резонанса
    fn check_parametric(&self, parametric_zone: Vec<Vec<(f64,f64)>>, current_speed: f64, course_angle: f64) -> bool {
        for cluster in parametric_zone {
            if self.point_in_polygon((course_angle, current_speed), &cluster) {
                return true;
            }
        }
        false
    }
    ///
    /// Проверка на попадание в зону брочинга
    fn check_broching(&self, broching_zone: Vec<(f64,f64)>, current_speed: f64, course_angle: f64) -> bool {
        self.point_in_polygon((course_angle, current_speed), &broching_zone)
    }
    ///
    /// Проверка на попадание в зону высоких волн
    fn check_high_waves(&self, high_waves_zone: Vec<(f64,f64)>, current_speed: f64, course_angle: f64) -> bool {
        self.point_in_polygon((course_angle, current_speed), &high_waves_zone)
    }
}
//
//
impl Eval<(), EvalResult> for HittingZonesEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let current_speed = ContextReadRef::<InitialCtx>::read_ref(&ctx).current_speed;
                let wave_heading_angle = ContextReadRef::<InitialCtx>::read_ref(&ctx).wave_heading_angle;
                let main_zone =  ContextRead::<MainResonantZoneSpeedFilterCtx>::read(&ctx).main_resonant_zone_speed_filter.clone();
                let mut main_zone_cartesian = Vec::new();
                for zone in main_zone.clone() {
                    let mut cartes_cluster: Vec<(f64,f64)> = Vec::new();
                    for point in zone {
                        let cartes_point = GrahamScan::polar_to_cartesian(point.0, point.1);
                        cartes_cluster.push(cartes_point);
                    }
                    main_zone_cartesian.push(cartes_cluster);
                }
                let parametric_zone =  ContextRead::<ParametricResonantZoneSpeedFilterCtx>::read(&ctx).parametric_resonant_zone_speed_filter.clone();
                let mut parametric_zone_cartesian = Vec::new();
                for zone in parametric_zone.clone() {
                    let mut cartes_cluster: Vec<(f64,f64)> = Vec::new();
                    for point in zone {
                        let cartes_point = GrahamScan::polar_to_cartesian(point.0, point.1);
                        cartes_cluster.push(cartes_point);
                    }
                    parametric_zone_cartesian.push(cartes_cluster);
                }
                let broching_zone =  ContextRead::<MoveBrochingFilterCtx>::read(&ctx).move_broching_filter.clone();
                let mut broching_zone_cartesian = Vec::new();
                for point in broching_zone.clone() {
                    let cartes_point = GrahamScan::polar_to_cartesian(point.0, point.1);
                    broching_zone_cartesian.push(cartes_point);
                }
                let impacts_high_waves =  ContextRead::<ImpactsHighWavesCtx>::read(&ctx).impacts_high_waves.clone();
                let mut impacts_high_waves_cartesian = Vec::new();
                for point in impacts_high_waves.clone() {
                    let cartes_point = GrahamScan::polar_to_cartesian(point.0, point.1);
                    impacts_high_waves_cartesian.push(cartes_point);
                }
                let ship_coord = GrahamScan::polar_to_cartesian(wave_heading_angle, current_speed);
                let result = HashMap::from([
                        ("Main".to_string(), self.check_main(main_zone_cartesian, ship_coord.1, ship_coord.0)),
                        ("Parametric".to_string(), self.check_parametric(parametric_zone_cartesian, ship_coord.1, ship_coord.0)),
                        ("Broaching".to_string(), self.check_broching(broching_zone_cartesian, ship_coord.1, ship_coord.0)),
                        ("HighWaves".to_string(), self.check_high_waves(impacts_high_waves_cartesian, ship_coord.1, ship_coord.0)),
                ]);
                let result = HittingZonesCtx {
                    hitting_zones: result,
                };
                ctx.write(result)
            }
            Err(err) => Err(err),
        }
    }
}


