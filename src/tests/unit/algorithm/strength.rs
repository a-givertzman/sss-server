#[cfg(test)]

use crate::algorithm::eval::*;
use crate::app::app::App;
use crate::conf::Conf;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::infrostructure::ApiClient;
use crate::kernel::{
    run::Run,
    types::{Arc, RwLock},
};
use sal_core::dbg::Dbg;
use sal_sync::thread_pool::{self, ThreadPool};
use crate::{algorithm::entities::ship_model::ship_model::ShipModel, infrostructure::{DevStream, SelectDevDoc, SelectDevInfo}, kernel::Eval, server::{Content, Cot, DevConf, DevStreamConf, QueryId, SelectAct, SelectContent, SelectCot, SelectReq, Server}};
use crate::algorithm::entities::{
    Bounds, model_cached::{self},
};
use crate::prelude::{Context, Initial, InitialCtx};
///
/// Application entry point
#[test]
fn strength() -> Result<(), Box<dyn std::error::Error>> {
    // let _log2 = log2::open("log.txt")
    //     .level(Logger::from_default_env().filter().as_str())
    //     .size(5 * 1024 * 1024)
    //     .rotate(10)
    //     .tee(false)
    //     .module(true)
    //     .start();


    DebugSession::new()
        .filter(LogLevel::Trace)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    
    let dbg = Dbg::own("main");
    let path = "config.yaml";
    let mut app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    }
    let conf = "./config.yaml";
    let conf = Conf::new(&dbg, conf);
    let ship_id = 2;
    let project_id = "NULL";
    let cache_dir = "src/assets/cache/sofia".into();
    let model_dir = "src/assets/model/sofia".into();
    let model_x = 65.25;
    let thread_pool = Arc::new(ThreadPool::new(&dbg, Some(conf.thread_pool.size)));  
    let mut dso_angles = vec![-60., -50., -40., -30., -12., 12., 30., 40., 50., 60.];
    dso_angles.append(&mut ((-11..=11).map(|v| (v as f64) * 5.).collect())); // -55, -50 .. 55
    dso_angles.append(&mut ((-8..=8).map(|v| v as f64).collect()));
    dso_angles.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    dso_angles.dedup();    
    let model_cached = model_cached::ModelCached::new(
        &dbg,
        model_cached::ModelCachedConf {
            model_dir,
            cache_dir,
            model_scale: 1000.,
            model_x,
            hull_heel_steps: vec![
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., 0., 2., 5.,
                10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            hull_trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., 0., 1., 2.,
                3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            compartment_heel_steps: vec![
                -60., -15., -5., 0., 5., 15., 60.,
            ],
            compartment_trim_steps: vec![
                -40., -10., -5., 0., 5., 10., 40.,
            ],
            ship_length_lbp: 130.5,
            draught_min: 2.001,
            hull_draught_min: 0.5,
            hull_draught_max: 14.,
            hull_draught_step: 0.5,
            bounds_level_step: 0.1,
            compartment_level_step_qnt: 10,
            dso_angles,
        },
        Arc::clone(&thread_pool),
    )
    .unwrap();
/*
    let res = model_cached.reload_shapes();            dbg!(&res);
 //   let res = model_cached.rebuild_caches();   dbg!(&res);
    let res = model_cached.rebuild_bounds(&bounds);    dbg!(&res);
  //  let res = model_cached.init();                     dbg!(&res);
  //  let res = model_cached.init_bounded(&bounds);      dbg!(&res);
    return Ok(());
*/    
    let api_client = Arc::new(ApiClient::new(
        &dbg,
        conf.api.address.database.clone(),
        conf.api.address.host.clone(),
        conf.api.address.port.clone(),
    ));
    let mut ship_model = ShipModel::new(
        &dbg,
        ship_id,
        project_id.to_owned(),
        model_cached,
        api_client.clone(),
    );
    let bounds = ship_model.init().unwrap();
    let ship_model = Arc::new(RwLock::new(ship_model));

    log::debug!("main | Calculations...");
    let ctx = 
    CriterionStabilityEval::new(
        &dbg,
        MetacentricHeightSubdivisionEval::new(
            &dbg,
            GrainEval::new(
                &dbg,
                CirculationEval::new(
                    &dbg,
                    AccelerationEval::new(
                        &dbg,
                        MinMetacentricHeightEval::new(
                            &dbg,
                            DSOAngleMaxEval::new(
                                &dbg,
                                DSOTimberMaxEval::new(
                                    &dbg,
                                    DSOIcingMaxEval::new(
                                        &dbg,
                                        DSOMaxEval::new(
                                            &dbg,
                                            DSOAreaEval::new(
                                                &dbg,
                                                StaticAngleEval::new(
                                                    &dbg,
                                                    WheatherEval::new(
                                                        &dbg,
                                                        RollingAmplitudeEval::new(
                                                            &dbg,
                                                            RollingPeriodEval::new(
                                                                &dbg,
                                                                WindEval::new(
                                                                    &dbg,
                                                                    WindageEval::new(
                                                                        &dbg,
                                                                        ship_model.clone(),
                                                                        LeverDiagramEval::new(
                                                                            &dbg,
                                                                            ship_model.clone(),
                                                                            //   link,
                                                                            MetacentricHeightEval::new(
                                                                                &dbg,
                                                                                // Before ZG
                                                                                UnitAreaEval::new(
                                                                                    &dbg,
                                                                                    StaticAreaEval::new(
                                                                                        &dbg,
                                                                                        ship_model.clone(),
                                                                                        BendingMomentEval::new(
                                                                                            &dbg,
                                                                                            ShearForceEval::new(
                                                                                                &dbg,
                                                                                                TotalForceEval::new(
                                                                                                    &dbg,          
                                                                                                    DynamicMassEval::new(
                                                                                                        &dbg,
                                                                                                        StrengthBalanceEval::new(
                                                                                                            &dbg,
                                                                                                            ship_model.clone(),
                                                                                                            StabilityBalanceEval::new(
                                                                                                                &dbg,
                                                                                                                ship_model.clone(),
                                                                                                                StaticMassEval::new(
                                                                                                                    &dbg,
                                                                                                                    WettingEval::new(
                                                                                                                        &dbg,
                                                                                                                        IcingEval::new(
                                                                                                                            &dbg,
                                                                                                                            StaticAreaEval::new(
                                                                                                                                &dbg,
                                                                                                                                ship_model.clone(),
                                                                                                                                IcingTimberEval::new(
                                                                                                                                    &dbg,
                                                                                                                                    IcingStabEval::new(
                                                                                                                                        &dbg,
                                                                                                                                        Initial::new(
                                                                                                                                            &dbg,                                                                                                                                            Arc::clone(&api_client),
                                                                                                                                            Context::new(InitialCtx::new(ship_id, project_id, bounds)),
                                                                                                                                        ),
                                                                                                                                    ),
                                                                                                                                ),
                                                                                                                            ),
                                                                                                                        ),
                                                                                                                    ),
                                                                                                                ),
                                                                                                            ),
                                                                                                        ),
                                                                                                    ),
                                                                                                ),
                                                                                            ),
                                                                                        ),
                                                                                    ),
                                                                                ),
                                                                            ),
                                                                        ),
                                                                    ),
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );

  /*  
    let result = DraftMarkEval::new(
        &dbg,
        CriterionDraughtEval::new(
            &dbg,
            ReserveBuoyncyEval::new(
                &dbg,
                ScrewEval::new(
                    &dbg,
                    BowBoardEval::new(
                        &dbg,
                        LoadLineEval::new(
                            &dbg,
                                ZgEval::new(
                                    thread_pool,
                                    &dbg,
                              //      &ship_model,
                                    ctx,
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    .eval(());*/

   // let initial: &InitialCtx = ctx.as_ref();
    ctx.eval(Zg::empty()).unwrap();
    
    Ok(())
}
