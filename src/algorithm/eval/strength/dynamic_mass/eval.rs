use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
        entities::{
            AddVec, Bound, Bounds,
            data::loads::{AssignmentType, UnitCargoType},
        },
        eval::{
            WettingCtx,
            strength::{DynamicMassCtx, IcingStrCtx, StrengthBalanceCtx},
        },
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, ContextWrite, InitialCtx},
};
use core::f64;
use sal_core::{dbg::Dbg, error::Error};
use std::collections::HashMap;

///
/// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
pub struct DynamicMassStrEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl DynamicMassStrEval {
    //
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "DynamicMassStrEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for DynamicMassStrEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let bounds = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                    .bounds
                    .clone()
                    .ok_or(error.err("initial error: no bounds!"))?;
                let unit = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                    .unit
                    .clone()
                    .ok_or(error.err("Read unit error: no data!"))?
                    .into_iter()
                    .filter(|v| v.mass > 0.)
                    .collect::<Vec<_>>();
                let (unit, bulkhead): (Vec<_>, Vec<_>) = unit
                    .into_iter()
                    .partition(|v| v.cargo_type != UnitCargoType::GrainBulkhead);
                let result = {
                    // распределения масс по типам
                    let (lightship_values, lightship_bounds): (Vec<_>, Vec<_>) =
                        <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                            .load_constant
                            .clone()
                            .ok_or(error.err("Read lightship error: no data!"))?
                            .data()
                            .into_iter()
                            .map(|v| (v.mass, Bound::Value(v.bound_x1, v.bound_x2)))
                            .unzip();
                    let lightship_bounds = Bounds::new(lightship_bounds)
                        .map_err(|err| error.pass_with("Bounds::new", err))?;
                    let bounded_hull = bounds
                        .intersect(&lightship_bounds, &lightship_values)
                        .map_err(|err| error.pass_with("bounds.intersect", err))?;
                    let bounded_equipment = vec![0.; bounds.len_qnt()]; //    TODO - сейчас в базе нет данных по equipment 
                    let bounded_icing = ContextRead::<IcingStrCtx>::read(&ctx).mass_values;
                    let bounded_wetting = ContextRead::<WettingCtx>::read(&ctx).mass_values;
                    let strength_balance: StrengthBalanceCtx = ctx.read();
                    // unit грузы представляются в виде прямоугольника, заполненного массой равномерно
                    let ballast: Vec<_> = unit
                        .iter()
                        .filter(|v| v.assigment_type == AssignmentType::Ballast)
                        .collect();
                    let stores: Vec<_> = unit
                        .iter()
                        .filter(|v| v.assigment_type == AssignmentType::Stores)
                        .collect();
                    let loads: Vec<_> = unit
                        .iter()
                        .filter(|v| v.assigment_type == AssignmentType::CargoLoad)
                        .collect();
                    let bounded_bulkhead: Vec<_> = bounds
                        .iter()
                        .map(|b| bulkhead.iter().fold(0., |s, v| s + v.mass(b).unwrap_or(0.)))
                        .collect();
                    let mut bounded_ballast: Vec<_> = bounds
                        .iter()
                        .map(|b| ballast.iter().fold(0., |s, v| s + v.mass(b).unwrap_or(0.)))
                        .collect();
                    let mut bounded_store: Vec<_> = bounds
                        .iter()
                        .map(|b| stores.iter().fold(0., |s, v| s + v.mass(b).unwrap_or(0.)))
                        .collect();
                    let mut bounded_load: Vec<_> = bounds
                        .iter()
                        .map(|b| loads.iter().fold(0., |s, v| s + v.mass(b).unwrap_or(0.)))
                        .collect();
                    // обработка результатов расчета распределения грузов по шпациям отсеков
                    let mut bounded_cargo = Vec::new();
                    let mut gaseous: Vec<_> = strength_balance
                        .gaseous
                        .iter()
                        .map(|v| (v.assigment_type, v.mass_values.clone())).collect();
                    bounded_cargo.append(&mut gaseous);
                    let mut bulk: Vec<_> = strength_balance
                        .bulk
                        .iter()
                        .map(|v| (v.assigment_type, v.mass_values.clone())).collect();
                    bounded_cargo.append(&mut bulk);
                    let mut liquid: Vec<_> = strength_balance
                        .liquid
                        .iter()
                        .map(|v| (v.assigment_type, v.mass_values.clone())).collect();
                    bounded_cargo.append(&mut liquid);
                    for (assigment_type, values) in bounded_cargo {
                        match assigment_type {
                            AssignmentType::Ballast => bounded_ballast
                                .add_vec(&values)
                                .map_err(|err| error.pass_with("vec_ballast.add", err))?,
                            AssignmentType::Stores => bounded_store
                                .add_vec(&values)
                                .map_err(|err| error.pass_with("vec_store.add", err))?,
                            AssignmentType::CargoLoad => bounded_load
                                .add_vec(&values)
                                .map_err(|err| error.pass_with("vec_cargo.add", err))?,
                            AssignmentType::Unspecified => (),
                        }
                    }
                    let mut mass_values = bounded_hull.clone();
                    mass_values
                        .add_vec(&bounded_equipment)
                        .map_err(|err| error.pass_with("mass_values.add vec_equipment", err))?;
                    mass_values
                        .add_vec(&bounded_bulkhead)
                        .map_err(|err| error.pass_with("mass_values.add vec_bulkhead", err))?;
                    mass_values
                        .add_vec(&bounded_ballast)
                        .map_err(|err| error.pass_with("mass_values.add vec_ballast", err))?;
                    mass_values
                        .add_vec(&bounded_store)
                        .map_err(|err| error.pass_with("mass_values.add vec_store", err))?;
                    mass_values
                        .add_vec(&bounded_load)
                        .map_err(|err| error.pass_with("mass_values.add vec_cargo", err))?;
                    mass_values
                        .add_vec(&bounded_icing)
                        .map_err(|err| error.pass_with("mass_values.add vec_icing", err))?;
                    mass_values
                        .add_vec(&bounded_wetting)
                        .map_err(|err| error.pass_with("mass_values.add vec_wetting", err))?;
                    /*             println!("value_mass_hull sum: {} result", vec_hull.iter().sum::<f64>()); // vec_hull.iter().for_each(|b| print!("{:.3} ", b));
                            println!("vec_equipment sum: {} result", vec_equipment.iter().sum::<f64>()); // vec_equipment.iter().for_each(|b| print!("{:.3} ", b));
                            println!("vec_bulkhead sum: {} result", vec_bulkhead.iter().sum::<f64>());  //vec_bulkhead.iter().for_each(|b| print!("{:.3} ", b));
                            println!("vec_ballast sum: {} result", vec_ballast.iter().sum::<f64>()); // vec_ballast.iter().for_each(|b| print!("{:.3} ", b));
                            println!("vec_store sum: {} result", vec_store.iter().sum::<f64>()); // vec_store.iter().for_each(|b| print!("{:.3} ", b));
                            println!("vec_cargo sum: {} result", vec_cargo.iter().sum::<f64>()); // vec_cargo.iter().for_each(|b| print!("{:.3} ", b));
                            println!("vec_icing sum: {} result", vec_icing.iter().sum::<f64>()); // vec_icing.iter().for_each(|b| print!("{:.3} ", b));
                            println!("vec_wetting sum: {} result", vec_wetting.iter().sum::<f64>()); // vec_wetting.iter().for_each(|b| print!("{:.3} ", b));
                    */
                    let mut data = HashMap::new();
                    data.insert("value_mass_hull".to_owned(), bounded_hull);
                    data.insert("value_mass_equipment".to_owned(), bounded_equipment);
                    data.insert("value_mass_bulkhead".to_owned(), bounded_bulkhead);
                    data.insert("value_mass_ballast".to_owned(), bounded_ballast);
                    data.insert("value_mass_store".to_owned(), bounded_store);
                    data.insert("value_mass_cargo".to_owned(), bounded_load);
                    data.insert("value_mass_icing".to_owned(), bounded_icing);
                    data.insert("value_mass_wetting".to_owned(), bounded_wetting);
                    data.insert("value_mass_sum".to_owned(), mass_values.clone());
                    log::info!(
                        "DynamicMass distr qnt:{} result:{}\n",
                        data.len(),
                        data.iter().fold(String::new(), |s, v| s + &format!(
                            "\n{}: {:.3}",
                            v.0,
                            v.1.iter().sum::<f64>()
                        ))
                    );
                    log::debug!(
                        "DynamicMass values:{}\n",
                        data.iter().fold(String::new(), |s, v| s + &format!(
                            "\n{}: {}",
                            v.0,
                            v.1.iter()
                                .fold(String::new(), |s, v| s + &format!("{:.3} ", v))
                        ))
                    );
                    DynamicMassCtx::new(mass_values, data)
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for DynamicMassStrEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval").field("dbg", &self.dbg).finish()
    }
}
