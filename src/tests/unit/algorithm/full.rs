use std::path::PathBuf;

use crate::{algorithm::eval::{
        criterion::{
            CriterionDraughtEval, CriterionStabilityEval, acceleration::eval::AccelerationEval, bow_board::eval::BowBoardEval, circulation::eval::CirculationEval, dso_angle_max::eval::DSOAngleMaxEval, dso_area::eval::DSOAreaEval, dso_icing_max::eval::DSOIcingMaxEval, dso_max::eval::DSOMaxEval, dso_timber_max::eval::DSOTimberMaxEval, eval::ResultCriterionEval, grain::eval::GrainEval, load_line::eval::LoadLineEval, metacentric_height_subdivision::eval::MetacentricHeightSubdivisionEval, min_metacentric_height::eval::MinMetacentricHeightEval, reserve_buoyncy::eval::ReserveBuoyncyEval, screw::eval::ScrewEval, static_angle::eval::StaticAngleEval, wheather::eval::WheatherEval
        }, draft_mark::eval::DraftMarkEval, icing_coeff::eval::IcingCoeffEval, icing_timber::eval::IcingTimberEval, icing_timber_bound::eval::IcingTimberBoundEval, seakeeping::eval::{apparent_frequencies::apparent_frequencies_eval::ApparentFrequenciesEval, main_resonant_zone::main_resonant_zone_eval::MainResonantZoneEval, main_resonant_zone_speed_filter::main_resonant_zone_speed_filter_eval::MainResonantZoneSpeedFilterEval, parametric_resonant_zone::parametric_resonant_zone_eval::ParametricResonantZoneEval, parametric_resonant_zone_speed_filter::parametric_resonant_zone_speed_filter_eval::ParametricResonantZoneSpeedFilterEval, period_excitement::period_excitement_eval::PeriodExcitementEval}, stability::{
            balance::eval::StabilityBalanceEval, dynamic_mass::eval::DynamicMassStabEval, icing::eval::IcingStabEval, lever_diagram::eval::LeverDiagramEval, metacentric_height::eval::MetacentricHeightEval, roll_amplitude::eval::RollingAmplitudeEval, roll_period::eval::RollingPeriodEval, static_mass::eval::StaticMassStabEval, wind::eval::WindEval, windage::eval::WindageEval
        }, strength::{
            area::eval::AreaStrEval, balance::eval::StrengthBalanceEval, dynamic_mass::eval::DynamicMassStrEval, icing::eval::IcingStrEval, result::eval::ResultStrEval, static_mass::eval::StaticMassStrEval,
        }, unit_area::eval::UnitAreaEval, wetting::eval::WettingEval, zg::eval::ZgEval        
    }};
use crate::app::app::App;
use crate::conf::Conf;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::infrostructure::ApiClient;
use crate::kernel::{
    run::Run,
    types::{Arc, RwLock},
};
use sal_core::dbg::Dbg;
use sal_sync::thread_pool::ThreadPool;
use crate::{algorithm::entities::ship_model::ship_model::ShipModel, infrostructure::{DevStream, SelectDevDoc, SelectDevInfo}, kernel::Eval, server::{Content, Cot, DevConf, DevStreamConf, QueryId, SelectAct, SelectContent, SelectCot, SelectReq, Server}};
use crate::algorithm::entities::model_cached::{self};
use crate::prelude::{Context, Initial, InitialCtx};
///
/// Application entry point
#[test]
fn full() -> Result<(), Box<dyn std::error::Error>> {
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
    let ship_id = conf.api.params.ship_id.clone();
    let project_id = conf.api.params.project_id.clone();
    let model_x = conf.api.model.midel_x;
    let model_name = conf.api.model.name.clone();
    let cache_dir: PathBuf = ("assets/cache/".to_owned() + &model_name).into();
    let model_dir: PathBuf = ("assets/model/".to_owned() + &model_name).into();
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
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., -1., -0.5, -0.2, 0., 
                0.2, 0.5, 1., 2., 5., 10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            hull_trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., -0.5, -0.2, 0., 
                0.2, 0.5, 1., 2., 3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            compartment_heel_steps: vec![
                -60., -30., -15., -10., -5., -2., 0., 2., 5., 10., 15., 30., 60.,
            ],
            compartment_trim_steps: vec![
                -40., -20., -10., -5., -2., 0., 2., 5., 10., 20., 40.,
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
        ship_id.to_owned(),
        project_id.to_owned(),
        model_cached,
        api_client.clone(),
    );
    ship_model.init().unwrap();
    let bounds = ship_model.bounds().unwrap();
    let ship_model = Arc::new(RwLock::new(ship_model));
    log::debug!("main | Calculations...");
    let ctx =        
        // criterion   
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
        // stability after ZG
        RollingAmplitudeEval::new(
            &dbg,
            RollingPeriodEval::new(
                &dbg,
                WindEval::new(
                    &dbg,
                    WindageEval::new(
                        &dbg,
                        Arc::clone(&ship_model),
                        LeverDiagramEval::new(
                            &dbg,
                            Arc::clone(&api_client),
                            Arc::clone(&ship_model),
                            MetacentricHeightEval::new(
                                &dbg,
        // strength
  /*      BendingMomentEval::new(
            &dbg,
            ShearForceEval::new(
                &dbg,
                TotalForceEval::new(
                    &dbg,                                                        
    */                
                ResultStrEval::new(
                    &dbg,  
                    Arc::clone(&api_client),
                    DynamicMassStrEval::new(
                        &dbg,
                        StrengthBalanceEval::new(
                            &dbg,
                            Arc::clone(&ship_model),                                
                            StaticMassStrEval::new(
                                &dbg,                                   
                                IcingStrEval::new(
                                    &dbg, 
                                    Arc::clone(&ship_model),                                      
                                    AreaStrEval::new(
                                        &dbg,
                                        Arc::clone(&ship_model),
        // stability before ZG
        DynamicMassStabEval::new(
            &dbg,
            StabilityBalanceEval::new(
                &dbg,
                Arc::clone(&api_client),
                Arc::clone(&ship_model),
                StaticMassStabEval::new(
                    &dbg,
                    IcingStabEval::new(
                        &dbg,
                        Arc::clone(&ship_model),                                        
        WettingEval::new(
            &dbg,
            IcingTimberEval::new(
                &dbg,
                IcingTimberBoundEval::new(
                    &dbg,
                    IcingCoeffEval::new(
                        &dbg,
                        UnitAreaEval::new(
                            &dbg,
                            Initial::new(
                                &dbg,
                                Arc::clone(&api_client),
                                Context::new(InitialCtx::new(&ship_id, &project_id, bounds)),
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
    //        ),
   //     ),
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
        );//.eval(Zg::empty());
        //
        let ctx = 
            ResultCriterionEval::new(
                &dbg,
                Arc::clone(&api_client),
                DraftMarkEval::new(
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
                                            Arc::clone(&thread_pool),
                                            &dbg,
                                            ctx,
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );  
       
        let ctx = ParametricResonantZoneSpeedFilterEval::new(
                &dbg,
                MainResonantZoneSpeedFilterEval::new(
                    &dbg,
                    ApparentFrequenciesEval::new(
                        &dbg,
                        PeriodExcitementEval::new(
                            &dbg,
                            MainResonantZoneEval::new(
                                &dbg,
                                ParametricResonantZoneEval::new(&dbg, ctx),
                            ),
                        ),
                    ),
                ),
            );

    ctx.eval(()).unwrap();   
    Ok(())
}
