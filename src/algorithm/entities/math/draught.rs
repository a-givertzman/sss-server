//! Расчет [осадки в произвольной точке](https://github.com/a-givertzman/sss/blob/master/design/algorithm/part03_draft/chapter01_floatingPosition/chapter02_draftPoint.md)
use sal_core::{dbg::Dbg, error::Error};

use crate::{
    algorithm::{entities::Position, eval::parameters::ParameterID},
    prelude::{Context, ContextParamsRead, ContextReadRef, InitialCtx},
};
//
pub struct Draught {
    midel_x: f64,
    draught_mid: f64,
    tg_t: f64,
    tg_h: f64,
    cos_h: f64,
}
//
impl Draught {
    //
    pub fn new(parent: impl Into<String>, ctx: &Context) -> Result<Self, Error> {
        let dbg = Dbg::new(parent, "Draught");
        let error = Error::new(&dbg, "new");
        let initial: &InitialCtx = ctx.read_ref();
        let ship_parameters = initial.ship_parameters.as_ref().unwrap();
        let midel_x = *ship_parameters
            .get("X midship from Fr0")
            .ok_or(error.err("Nomidship in ship_parameters"))?;
        let heel = ctx.read_params(ParameterID::Roll);
        let trim = ctx.read_params(ParameterID::TrimDeg);
        let draught_mid = ctx.read_params(ParameterID::DraughtMid);
        let heel_r = heel.to_radians();
        let trim_r = trim.to_radians();        
        let tg_t = trim_r.tan();
        let tg_h = heel_r.tan();
        let cos_h = heel_r.cos();
        //println!("midel_x:{midel_x} heel:{heel} trim:{trim} heel_r:{heel_r} trim_r:{trim_r} draught_mid:{draught_mid} tg_h:{tg_h} tg_t:{tg_t} cos_h:{cos_h}");
        Ok(Self {
            midel_x,
            draught_mid,
            tg_t,
            tg_h,
            cos_h,
        })
    }
    //
    pub fn value(&self, p: &Position) -> f64 {
        let d_zi = p.y() * self.tg_h + (p.x() - self.midel_x) * self.tg_t / self.cos_h;
        let z_fix = self.draught_mid + d_zi;
      //  println!("p:{} d_zi:{d_zi} z_fix:{z_fix}", p.print());
        z_fix
    }
}
