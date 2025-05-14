use crate::{
    algorithm::{context::context_access::ContextReadRef, entities::data::loads::{AssignmentType, UnitCargoType}},
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ship_model::model_link::Link,
    ContextWrite,
};
use sal_core::{dbg::Dbg, error::Error};
use super::mass_ctx::MassCtx;

///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct MassEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl MassEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "MassEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for MassEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let bounds = match initial.bounds.clone() {
                    Some(data) => data,
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read bounds error: no data!",
                            self.dbg
                        )))
                    }
                };
                let ship_parameters = match initial.ship_parameters.as_ref() {
                    Some(data) => data,
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read voyage error: no data!",
                            self.dbg
                        )))
                    }
                };
                let const_mass_shift_x = match ship_parameters.get("LCG from middle") {
                    Some(data) => *data,
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read const_mass_shift_x error: no data!",
                            self.dbg
                        )))
                    }
                };
                let const_mass_shift_y = match ship_parameters.get("TCG from CL") {
                    Some(data) => *data,
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read const_mass_shift_y error: no data!",
                            self.dbg
                        )))
                    }
                };
                let const_mass_shift_z = match ship_parameters.get("VCG from BL") {
                    Some(data) => *data,
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read const_mass_shift_z error: no data!",
                            self.dbg
                        )))
                    }
                };
                let load_constant = match initial.load_constant.clone() {
                    Some(data) => data.data(),
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read load_constant error: no data!",
                            self.dbg
                        )))
                    }
                };
                let unit: Vec<_> = match initial.unit.as_ref() {
                    Some(data) => data.data(),
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read unit error: no data!",
                            self.dbg
                        )))
                    }
                };
                let liquid: Vec<_> = match initial.liquid.as_ref() {
                    Some(data) => data.data(),
                    None => {
                        return Err(Error(format!(
                            "{}.eval | Read liquid error: no data!",
                            self.dbg
                        )))
                    }
                };
                let mut vec_hull = Vec::new();
                //    let mut vec_equipment = Vec::new();
                let mut vec_bulkhead = Vec::new();
                let mut vec_ballast = Vec::new();
                let mut vec_store = Vec::new();
                let mut vec_cargo = Vec::new();
                let mut vec_icing = Vec::new();
                let mut vec_wetting = Vec::new();
                let mut vec_sum = Vec::new();
                let mut res: Vec<f64> = Vec::new();
                for (i, b) in bounds.iter().enumerate() {
                    let mut hull = load_constant.iter().map(|v| v.mass(b)).sum();
                    let mut equipment = 0.;
                    /*         for v in self
                        .loads_const
                        .iter()
                        .filter(|v| v.load_type() == LoadingType::Equipment)
                    {
                        equipment += v.value(b)?;
                    }
                    vec_equipment.push(equipment);*/
                    let (bulkhead, bulkhead_error): (Vec<_>, Vec<_>)  = unit.iter()
                        .filter(|v| v.cargo_type == UnitCargoType::GrainBulkhead)
                        .map(|v| v.mass(b)).partition(Result::is_ok);
                    vec_bulkhead.push(bulkhead.iter().map(|v| v.unwrap()).sum());
                    let (ballast, ballast_error): (Vec<_>, Vec<_>) = liquid
                        .iter()
                        .filter(|v| v.assigment_type== AssignmentType::Ballast)
                        .map(|v| v.mass(b)).partition(Result::is_ok);
                    vec_ballast.push(ballast.iter().map(|v| v.unwrap()).sum());
                    let (ballast, ballast_error): (Vec<_>, Vec<_>) = unit
                        .iter()
                        .filter(|v| v.assigment_type== AssignmentType::Stores)
                        .map(|v| v.mass(b)).partition(Result::is_ok);
                    vec_store.push(ballast.iter().map(|v| v.unwrap()).sum());

                    let mut cargo = 0.;
                    for v in self
                        .loads_variable
                        .iter()
                        .filter(|v| v.load_type() == LoadingType::Cargo)
                    {
                        cargo += v.value(b)?;
                    }
                    vec_cargo.push(cargo);
                    let icing = self.icing_mass.mass(b)?;
                    vec_icing.push(icing);
                    let wetting = self.wetting_mass.mass(b)?;
                    vec_wetting.push(wetting);
                    res.push(
                        hull + equipment + bulkhead + ballast + store + cargo + icing + wetting,
                    );
                }
                vec_hull.push(vec_hull.iter().sum());
                vec_equipment.push(vec_equipment.iter().sum());
                vec_bulkhead.push(vec_bulkhead.iter().sum());
                vec_ballast.push(vec_ballast.iter().sum());
                vec_store.push(vec_store.iter().sum());
                vec_cargo.push(vec_cargo.iter().sum());
                vec_icing.push(vec_icing.iter().sum());
                vec_wetting.push(vec_wetting.iter().sum());
                vec_sum.append(&mut res.clone());
                vec_sum.push(res.iter().sum());

                let result = StrengthAreaCtx {
                    area_v_array: area_v,
                    area_h: const_area_h,
                    area_timber_h,
                };
                ctx.write(result)
            }
            Err(err) => Err(Error(format!(
                "{}.eval | Read context error: {:?}",
                self.dbg, err
            ))),
        }
    }
}
//
//
impl std::fmt::Debug for MassEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
