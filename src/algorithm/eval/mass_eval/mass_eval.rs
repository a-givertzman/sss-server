use std::collections::HashMap;

use super::mass_ctx::MassCtx;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
        entities::{
            Bound, Bounds,
            data::loads::{AssignmentType, UnitCargoType},
        },
        eval::{BalanceCtx, IcingCtx, WettingCtx, parameters::ParameterID},
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{ContextParamsWrite, ContextRead, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
pub struct MassEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl MassEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
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
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let bounds = initial
                    .bounds
                    .clone()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let unit = initial
                    .unit
                    .clone()
                    .ok_or(error.err("Read unit error: no data!"))?
                    .into_iter()
                    .filter(|v| v.mass > 0.)
                    .collect::<Vec<_>>();
                let (unit, bulkhead): (Vec<_>, Vec<_>) = unit
                    .into_iter()
                    .partition(|v| v.cargo_type != UnitCargoType::GrainBulkhead);
                let (lightship_values, lightship_bounds): (Vec<_>, Vec<_>) = initial
                    .load_constant
                    .clone()
                    .ok_or(error.err("Read lightship error: no data!"))?
                    .data()
                    .into_iter()
                    .map(|v| (v.mass, Bound::Value(v.bound_x1, v.bound_x2)))
                    .unzip();
                {
                    // суммарные массы по типам
                    let gaseous = initial
                        .gaseous
                        .as_ref()
                        .ok_or(error.err("Read gaseous error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                    let bulk = initial
                        .bulk
                        .as_ref()
                        .ok_or(error.err("Read bulk error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                    let liquid = initial
                        .liquid
                        .as_ref()
                        .ok_or(error.err("Read liquid error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                    let mass = |assigment_type: AssignmentType| {
                        let gaseous = gaseous
                            .iter()
                            .filter(|v| v.assigment_type == assigment_type)
                            .fold(0., |sum, v| sum + v.mass);
                        let bulk = bulk
                            .iter()
                            .filter(|v| v.assigment_type == assigment_type)
                            .fold(0., |sum, v| sum + v.mass);
                        let liquid = liquid
                            .iter()
                            .filter(|v| v.assigment_type == assigment_type)
                            .fold(0., |sum, v| sum + v.mass);
                        let unit = unit
                            .iter()
                            .filter(|v| v.assigment_type == assigment_type)
                            .fold(0., |sum, v| sum + v.mass);
                //        dbg!(assigment_type, gaseous, bulk, liquid, unit);
                        gaseous + bulk + liquid + unit
                    };
                    let ballast = mass(AssignmentType::Ballast);
                    let stores = mass(AssignmentType::Stores);
                    let cargo = mass(AssignmentType::CargoLoad);
                    let bulkhead = bulkhead.iter().fold(0., |sum, v| sum + v.mass);
                    let deadweight = ballast + stores + cargo + bulkhead; // Суммарная масса переменного груза
                    let lightship = lightship_values.iter().fold(0., |sum, v| sum + v);
                    let icing = ContextRead::<IcingCtx>::read(&ctx).mass;
                    let wetting = ContextRead::<WettingCtx>::read(&ctx).mass;
                    let mass_sum = deadweight + lightship + wetting + icing;
                    ctx.write_params(ParameterID::Displacement, mass_sum);
                    ctx.write_params(ParameterID::MassBallast, ballast);
                    ctx.write_params(ParameterID::MassStores, stores);
                    ctx.write_params(ParameterID::MassBulkhead, bulkhead);
                    ctx.write_params(ParameterID::MassCargo, cargo);
                    ctx.write_params(ParameterID::MassDeadweight, deadweight);
                    ctx.write_params(ParameterID::MassLightship, lightship);
                    ctx.write_params(ParameterID::MassIcing, icing);
                    ctx.write_params(ParameterID::MassWetting, wetting);
                    log::info!(
                        "\t Mass ballast:{ballast}, stores:{stores}, bulkhead:{bulkhead}
                        cargo:{cargo}, deadweight:{deadweight}, lightship:{lightship},
                        icing:{icing}, wetting:{wetting} sum:{mass_sum}"
                    );
               /*     println!(
                        "\t Mass ballast:{ballast}, stores:{stores}, bulkhead:{bulkhead}
                        cargo:{cargo}, deadweight:{deadweight}, lightship:{lightship},
                        icing:{icing}, wetting:{wetting} sum:{mass_sum}"
                    );*/
                }
                let result = {
                    // распределения масс по типам
                    let balance: BalanceCtx = ctx.read();
                    let mut vec_hull = bounds
                        .intersect(
                            &Bounds::new(lightship_bounds)
                                .map_err(|err| error.pass_with("Bounds::new", err))?,
                            &lightship_values,
                        )
                        .map_err(|err| error.pass_with("bounds.intersect", err))?;
                    let mut vec_equipment = vec![0.; bounds.len_qnt()]; //    TODO - сейчас в базе нет данных по equipment 
                    let mut vec_bulkhead = Vec::new();
                    let mut vec_ballast = Vec::new();
                    let mut vec_store = Vec::new();
                    let mut vec_cargo = Vec::new();
                    let mut vec_icing = ContextRead::<IcingCtx>::read(&ctx).mass_values;
                    let mut vec_wetting = ContextRead::<WettingCtx>::read(&ctx).mass_values;
                    // unit грузы представляются в виде прямоугольника, заполненного массой равномерно
                    for b in bounds.iter() {
                        vec_bulkhead
                            .push(bulkhead.iter().fold(0., |s, v| s + v.mass(b).unwrap_or(0.)));
                        vec_ballast.push(
                            unit.iter()
                                .filter(|v| v.assigment_type == AssignmentType::Ballast)
                                .fold(0., |s, v| s + v.mass(b).unwrap_or(0.)),
                        );
                        vec_store.push(
                            unit.iter()
                                .filter(|v| v.assigment_type == AssignmentType::Stores)
                                .fold(0., |s, v| s + v.mass(b).unwrap_or(0.)),
                        );
                        vec_cargo.push(
                            unit.iter()
                                .filter(|v| v.assigment_type == AssignmentType::CargoLoad)
                                .fold(0., |s, v| s + v.mass(b).unwrap_or(0.)),
                        );
                    }
                    let add = |v1: &Vec<f64>, v2: &Vec<f64>| -> Vec<f64> {
                        v1.iter().zip(v2.iter()).map(|(v1, v2)| v1 + v2).collect()
                    };
                    let mut process_by_type =
                        |values: &Vec<f64>, assigment_type: AssignmentType| match assigment_type {
                            AssignmentType::Ballast => vec_ballast = add(&vec_ballast, &values),
                            AssignmentType::Stores => vec_store = add(&vec_store, &values),
                            AssignmentType::CargoLoad => vec_cargo = add(&vec_cargo, &values),
                            AssignmentType::Unspecified => (),
                        };
                    for v in balance.gaseous {
                        process_by_type(&v.mass_values, v.assigment_type);
                    }
                    for v in balance.bulk {
                        process_by_type(&v.mass_values, v.assigment_type);
                    }
                    for v in balance.liquid {
                        process_by_type(&v.mass_values, v.assigment_type);
                    }
                    let mass_values = add(
                        &add(
                            &add(
                                &add(
                                    &add(
                                        &add(&add(&vec_hull, &vec_equipment), &vec_bulkhead),
                                        &vec_ballast,
                                    ),
                                    &vec_store,
                                ),
                                &vec_cargo,
                            ),
                            &vec_icing,
                        ),
                        &vec_wetting,
                    );
                    vec_hull.push(vec_hull.iter().sum());
                    vec_equipment.push(vec_equipment.iter().sum());
                    vec_bulkhead.push(vec_bulkhead.iter().sum());
                    vec_ballast.push(vec_ballast.iter().sum());
                    vec_store.push(vec_store.iter().sum());
                    vec_cargo.push(vec_cargo.iter().sum());
                    vec_icing.push(vec_icing.iter().sum());
                    vec_wetting.push(vec_wetting.iter().sum());
                    let mut vec_sum = mass_values.clone();
                    vec_sum.push(vec_sum.iter().sum());
                    log::info!("\t Mass values:{:?} ", mass_values);
                  //  dbg!("\t Mass values:{:?} ", mass_values);
                    let mut data = HashMap::new();
                    data.insert("value_mass_hull".to_owned(), vec_hull);
                    data.insert("value_mass_equipment".to_owned(), vec_equipment);
                    data.insert("value_mass_bulkhead".to_owned(), vec_bulkhead);
                    data.insert("value_mass_ballast".to_owned(), vec_ballast);
                    data.insert("value_mass_store".to_owned(), vec_store);
                    data.insert("value_mass_cargo".to_owned(), vec_cargo);
                    data.insert("value_mass_icing".to_owned(), vec_icing);
                    data.insert("value_mass_wetting".to_owned(), vec_wetting);
                    data.insert("value_mass_sum".to_owned(), vec_sum);
                    MassCtx::new(data, mass_values)
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for MassEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval").field("dbg", &self.dbg).finish()
    }
}
