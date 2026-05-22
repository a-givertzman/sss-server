use sal_3dlib_core::math::Position;
use sal_core::{dbg::Dbg, error::Error};
use crate::{
    algorithm::eval::parameters::ParameterID,
    prelude::{Context, ContextParamsRead, ContextReadRef, InitialCtx},
};

//
pub struct Draught {
    draught: sal_3dlib_core::math::Draught,
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
        Ok(Self { draught: sal_3dlib_core::math::Draught::new(
                heel, 
                trim, 
                midel_x, 
                draught_mid,          
        )})
    }
    //
    pub fn value(&self, p: &Position) -> f64 {
        self.draught.value(p)
    }
}
