use crate::{
    algorithm::{
        context::context_access::ContextReadRef,
        entities::{
            AddVec, Bound, Bounds,
            data::loads::{AssignmentType, UnitCargoType},
        },
        eval::{WettingCtx, parameters::ParameterID, stability::IcingStabCtx, strength::{DynamicMassCtx, IcingStrCtx, StrengthBalanceCtx}},
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextParamsWrite, ContextRead, ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use core::f64;
use std::collections::HashMap;

///
/// Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
pub struct DynamicMassEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl DynamicMassEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "DynamicMassEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for DynamicMassEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(mut ctx) => {
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
                {
                    // суммарные массы по типам
                    let lightship = match <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .load_constant
                        .clone()
                    {
                        Some(data) => data.data().iter().map(|v| v.mass).sum(),
                        None => return Err(error.err("Read load_constant error: no data!")),
                    };
                    let gaseous = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .gaseous
                        .as_ref()
                        .ok_or(error.err("Read gaseous error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
             //       for v in &gaseous {  println!("gaseous {} mass:{} ", v.code, v.mass);   }
                    let bulk = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .bulk
                        .as_ref()
                        .ok_or(error.err("Read bulk error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
               //     for v in &bulk {   println!("bulk {} mass:{} ", v.code, v.mass);   }
                    let liquid = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .liquid
                        .as_ref()
                        .ok_or(error.err("Read liquid error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| v)
                        .collect::<Vec<_>>();
                //    for v in &liquid {   println!("liquid {} mass:{} ", v.code, v.mass);  }
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
                        gaseous + bulk + liquid + unit
                    };
                    let ballast = mass(AssignmentType::Ballast);
                    let stores = mass(AssignmentType::Stores);
                    let cargo = mass(AssignmentType::CargoLoad);
                    let bulkhead = bulkhead.iter().fold(0., |sum, v| sum + v.mass);
                    let deadweight = ballast + stores + cargo + bulkhead; // Суммарная масса переменного груза
                    let icing = ContextRead::<IcingStabCtx>::read(&ctx).p_ice;
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
            /*        println!(
                        "Mass 
                        ballast:{ballast}, 
                        stores:{stores}, 
                        bulkhead:{bulkhead}
                        cargo:{cargo}, 
                        deadweight:{deadweight}, 
                        lightship:{lightship},
                        icing:{icing}, 
                        wetting:{wetting} 
                        sum:{mass_sum}"
                    );*/
                }
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
                    let vec_hull = bounds
                        .intersect(&lightship_bounds, &lightship_values)
                        .map_err(|err| error.pass_with("bounds.intersect", err))?;
                    let vec_equipment = vec![0.; bounds.len_qnt()]; //    TODO - сейчас в базе нет данных по equipment 
                    let mut vec_bulkhead = Vec::new();
                    let mut vec_ballast = Vec::new();
                    let mut vec_store = Vec::new();
                    let mut vec_cargo = Vec::new();
                    let vec_icing = ContextRead::<IcingStrCtx>::read(&ctx).mass_values;
                    let vec_wetting = ContextRead::<WettingCtx>::read(&ctx).mass_values;
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
                    let mut process_by_type =
                        |values: &Vec<f64>, assigment_type: AssignmentType| match assigment_type {
                            AssignmentType::Ballast => vec_ballast
                                .add_vec(&values)
                                .map_err(|err| error.pass_with("vec_ballast.add", err)),
                            AssignmentType::Stores => vec_store
                                .add_vec(&values)
                                .map_err(|err| error.pass_with("vec_store.add", err)),
                            AssignmentType::CargoLoad => vec_cargo
                                .add_vec(&values)
                                .map_err(|err| error.pass_with("vec_cargo.add", err)),
                            AssignmentType::Unspecified => Ok(()),
                        };
                    let strength_balance: StrengthBalanceCtx = ctx.read();

              /*      let gaseous = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .gaseous
                        .as_ref()
                        .ok_or(error.err("Read gaseous error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| (v.code.clone(), v.mass))
                        .collect::<HashMap<_, _>>();
                    let bulk = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .bulk
                        .as_ref()
                        .ok_or(error.err("Read bulk error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| (v.code.clone(), v.mass))
                        .collect::<HashMap<_, _>>();
                    let liquid = <dyn ContextReadRef<InitialCtx>>::read_ref(&ctx)
                        .liquid
                        .as_ref()
                        .ok_or(error.err("Read bulk error: no data!"))?
                        .iter()
                        .filter(|(_, v)| v.mass > 0.)
                        .map(|(_, v)| (v.code.clone(), v.mass))
                        .collect::<HashMap<_, _>>();*/
                    for v in strength_balance.gaseous {
                //        println!("gaseous {} mass:{} vec_sum:{}", v.code, gaseous.get(&v.code).unwrap(), v.mass_values.iter().sum::<f64>());
                        process_by_type(&v.mass_values, v.assigment_type)?;
                    }
                    for v in strength_balance.bulk {
                 //       println!("bulk {} vec_sum:{}", v.code, v.mass_values.iter().sum::<f64>());
                 //       v.mass_values.iter().for_each(|b| print!("{:.3} ", b));
                        process_by_type(&v.mass_values, v.assigment_type)?;
                    }
                    for v in strength_balance.liquid {
                //        println!("liquid {} mass:{} vec_sum:{}", v.code, liquid.get(&v.code).unwrap(), v.mass_values.iter().sum::<f64>());
                        process_by_type(&v.mass_values, v.assigment_type)?;
                    }
                    let mut mass_values = vec_hull.clone();
                    mass_values
                        .add_vec(&vec_equipment)
                        .map_err(|err| error.pass_with("mass_values.add vec_equipment", err))?;
                    mass_values
                        .add_vec(&vec_bulkhead)
                        .map_err(|err| error.pass_with("mass_values.add vec_bulkhead", err))?;
                    mass_values
                        .add_vec(&vec_ballast)
                        .map_err(|err| error.pass_with("mass_values.add vec_ballast", err))?;
                    mass_values
                        .add_vec(&vec_store)
                        .map_err(|err| error.pass_with("mass_values.add vec_store", err))?;
                    mass_values
                        .add_vec(&vec_cargo)
                        .map_err(|err| error.pass_with("mass_values.add vec_cargo", err))?;
                    mass_values
                        .add_vec(&vec_icing)
                        .map_err(|err| error.pass_with("mass_values.add vec_icing", err))?;
                    mass_values
                        .add_vec(&vec_wetting)
                        .map_err(|err| error.pass_with("mass_values.add vec_wetting", err))?;                    
       /*             println!("value_mass_hull sum: {} result", vec_hull.iter().sum::<f64>()); // vec_hull.iter().for_each(|b| print!("{:.3} ", b));
                    println!("vec_equipment sum: {} result", vec_equipment.iter().sum::<f64>()); // vec_equipment.iter().for_each(|b| print!("{:.3} ", b));
                    println!("vec_bulkhead sum: {} result", vec_bulkhead.iter().sum::<f64>());  //vec_bulkhead.iter().for_each(|b| print!("{:.3} ", b));
                    println!("vec_ballast sum: {} result", vec_ballast.iter().sum::<f64>()); // vec_ballast.iter().for_each(|b| print!("{:.3} ", b));
                    println!("vec_store sum: {} result", vec_store.iter().sum::<f64>()); // vec_store.iter().for_each(|b| print!("{:.3} ", b));
                    println!("vec_cargo sum: {} result", vec_cargo.iter().sum::<f64>()); // vec_cargo.iter().for_each(|b| print!("{:.3} ", b));
                    println!("vec_icing sum: {} result", vec_icing.iter().sum::<f64>()); // vec_icing.iter().for_each(|b| print!("{:.3} ", b));
                    println!("vec_wetting sum: {} result", vec_wetting.iter().sum::<f64>()); // vec_wetting.iter().for_each(|b| print!("{:.3} ", b));
            */      let mut data = HashMap::new();
                    data.insert("value_mass_hull".to_owned(), vec_hull);
                    data.insert("value_mass_equipment".to_owned(), vec_equipment);
                    data.insert("value_mass_bulkhead".to_owned(), vec_bulkhead);
                    data.insert("value_mass_ballast".to_owned(), vec_ballast);
                    data.insert("value_mass_store".to_owned(), vec_store);
                    data.insert("value_mass_cargo".to_owned(), vec_cargo);
                    data.insert("value_mass_icing".to_owned(), vec_icing);
                    data.insert("value_mass_wetting".to_owned(), vec_wetting);
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
                            v.1.iter().fold(String::new(), |s, v| s + &format!(
                            "{:.3} ", v)
                        )))
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
impl std::fmt::Debug for DynamicMassEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MassEval").field("dbg", &self.dbg).finish()
    }
}
