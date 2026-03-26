use sal_core::{dbg::Dbg, error::Error};
use crate::{
    algorithm::eval::seakeeping::eval::{
        hitting_zones::hitting_point_ctx::HittingZonesCtx, 
        impacts_high_waves::impacts_high_waves_ctx::ImpactsHighWavesCtx, 
        main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_ctx::MainResonantZoneSpeedFilterCtx, 
        move_broching_filter::move_broching_filter_ctx::MoveBrochingFilterCtx, 
        parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_ctx::ParametricResonantZoneSpeedFilterCtx
    }, infrostructure::ApiClient, kernel::{Eval, types::{Arc, eval_result::EvalResult}}, prelude::{
        ContextRead, ContextReadRef, ContextWrite, InitialCtx
    }
};
///
/// Запись результатов [Seakeeping] в БД
pub struct SendSeakeepingResult {
    dbg: Dbg,
    api_client: Arc<ApiClient>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl SendSeakeepingResult {
    ///
    /// Новый экземпляр [SendSeakeepingResult]
    pub fn new(
        parent: impl Into<String>,
        api_client: Arc<ApiClient>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "SendSeakeepingResult");
        Self {
            dbg,
            api_client,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for SendSeakeepingResult {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
                let ship_id = ContextReadRef::<InitialCtx>::read_ref(&ctx).ship_id.clone();
                // отправка результатов вычисления резонансных зон
                log::info!("send_seakeeping begin");
                let mut full_sql = "DO $$ BEGIN ".to_owned();
                full_sql += &format!("DELETE FROM seakeeping_zones WHERE ship_id={ship_id};");
                full_sql += " INSERT INTO seakeeping_zones (ship_id, angle, speed, zone_id, subzone) VALUES";
                let mut subzone = 1;
                let parametric_zone = ContextRead::<ParametricResonantZoneSpeedFilterCtx>::read(&ctx).parametric_resonant_zone_speed_filter.clone();
                parametric_zone.iter().for_each(|points| {
                    let subzone_type = format!("SubParametricZone{}", subzone);
                    for point in points {
                        let angle = point.0;
                        let speed = point.1;
                        full_sql += &format!(" ({ship_id}, {}, {}, 'Parametric', '{}'),", angle, speed, subzone_type);
                    }
                    subzone += 1;
                });
                let main_zone = ContextRead::<MainResonantZoneSpeedFilterCtx>::read(&ctx).main_resonant_zone_speed_filter.clone();
                let mut subzone = 1;
                main_zone.iter().for_each(|points| {
                    let subzone_type = format!("SubMainZone{}", subzone);
                    for point in points {
                        let angle = point.0;
                        let speed = point.1;
                        full_sql += &format!(" ({ship_id}, {}, {}, 'Main', '{}'),", angle, speed, subzone_type);
                    }
                    subzone += 1;
                });
                let broching_zone = ContextRead::<MoveBrochingFilterCtx>::read(&ctx).move_broching_filter.clone();
                broching_zone.iter().for_each(|point| {
                    let angle = point.0;
                    let speed = point.1;
                    full_sql += &format!(" ({ship_id}, {}, {}, 'Broching', 'None'),", angle, speed);
                });
                let highwaves_zone = ContextRead::<ImpactsHighWavesCtx>::read(&ctx).impacts_high_waves.clone();
                highwaves_zone.iter().for_each(|point| {
                    let angle = point.0;
                    let speed = point.1;
                    full_sql += &format!(" ({ship_id}, {}, {}, 'HighWaves', 'None'),", angle, speed);
                });
                full_sql.remove(full_sql.len()-1);
                full_sql.push(';');
                full_sql += " END$$;";
                self.api_client.fetch(&full_sql)?;
                // отправка резульатов расчёта попадания в резонансные зоны
                let mut full_sql = "DO $$ BEGIN ".to_owned();
                full_sql += &format!("DELETE FROM seakeeping_zones_status WHERE ship_id={ship_id};");
                full_sql += " INSERT INTO seakeeping_zones_status (ship_id, zone_id, zone_status) VALUES";
                let zones_status = ContextRead::<HittingZonesCtx>::read(&ctx).hitting_zones.clone();
                zones_status.iter().for_each(|zone_status| {
                    let zone_name = zone_status.0;
                    let zone_status = zone_status.1;
                    full_sql += &format!(" ({ship_id}, '{zone_name}', {zone_status}),")
                });
                full_sql.remove(full_sql.len()-1);
                full_sql.push(';');
                full_sql += " END$$;";
                self.api_client.fetch(&full_sql)?;
                log::info!("send_seakeeping end");
                ctx.write(())                
            }
            Err(err) => Err(err),
        }
    }
}