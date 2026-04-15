use std::{collections::HashMap, path::PathBuf};
use crate::algorithm::entities::Bounds;
use crate::app::app::App;
use crate::conf::Conf;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::kernel::{
    run::Run,
    types::Arc,
};
use sal_core::dbg::Dbg;
use sal_sync::thread_pool::ThreadPool;
use crate::algorithm::entities::model_cached::{self};
///
/// Application entry point
#[test]
fn build_caches() -> Result<(), Box<dyn std::error::Error>> {
    DebugSession::new()
        .filter(LogLevel::Trace)
        .module("api_tools", LogLevel::Error)
        .module("sal_sync", LogLevel::Error)
        .module("ena", LogLevel::Error)
        .init();
    let dbg = Dbg::own("test build_caches");
    let path = "config.yaml";
    let mut app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    }
    let conf = "./config.yaml";
    let conf = Conf::new(&dbg, conf);
    let model_x = conf.api.model.midel_x;
    let model_name = conf.api.model.name.clone();
    let physical_frames: [f64; 196] = [
        -3.6, -3.0, -2.4, -1.8, -1.2, -0.6, 0.0, 0.6, 1.2, 1.8, 2.4, 3.0, 3.6, 4.2, 4.8, 5.4, 6.0, 6.7,
        7.4, 8.1, 8.8, 9.5, 10.2, 10.9, 11.6, 12.3, 13.0, 13.7, 14.4, 15.1, 15.8, 16.5, 17.2, 17.9,
        18.6, 19.34, 20.08, 20.82, 21.56, 22.3, 23.04, 23.78, 24.52, 25.26, 26.0, 26.74, 27.48, 28.22,
        28.96, 29.7, 30.44, 31.18, 31.92, 32.66, 33.4, 34.14, 34.88, 35.62, 36.36, 37.1, 37.84, 38.58,
        39.32, 40.06, 40.80, 41.54, 42.28, 43.02, 43.76, 44.5, 45.24, 45.98, 46.72, 47.46, 48.2, 48.94,
        49.68, 50.42, 51.16, 51.9, 52.64, 53.38, 54.12, 54.86, 55.6, 56.34, 57.08, 57.82, 58.56, 59.30,
        60.04, 60.78, 61.52, 62.26, 63.0, 63.74, 64.48, 65.22, 65.96, 66.7, 67.44, 68.18, 68.92, 69.66,
        70.4, 71.14, 71.88, 72.62, 73.36, 74.1, 74.84, 75.58, 76.32, 77.06, 77.8, 78.54, 79.28, 80.02,
        80.76, 81.5, 82.24, 82.98, 83.72, 84.46, 85.2, 85.94, 86.68, 87.42, 88.16, 88.9, 89.64, 90.38,
        91.12, 91.86, 92.6, 93.34, 94.08, 94.82, 95.56, 96.3, 97.04, 97.78, 98.52, 99.26, 100.0,
        100.74, 101.48, 102.22, 102.96, 103.7, 104.44, 105.18, 105.92, 106.66, 107.4, 108.14, 108.88,
        109.62, 110.36, 111.1, 111.84, 112.58, 113.32, 114.06, 114.8, 115.54, 116.28, 117.02, 117.76,
        118.5, 119.24, 119.98, 120.72, 121.46, 122.2, 122.94, 123.68, 124.42, 125.16, 125.9, 126.5,
        127.1, 127.7, 128.3, 128.9, 129.5, 130.1, 130.7, 131.3, 131.9, 132.5, 133.1, 133.7, 134.3,
        134.9, 135.5,
    ];   
    let bounds = Bounds::from_array(&physical_frames, 0.).unwrap();
    let cache_dir: PathBuf = ("assets/cache/".to_owned() + &model_name).into();
    let model_dir: PathBuf = ("assets/model/".to_owned() + &model_name).into();
    let thread_pool = Arc::new(ThreadPool::new(&dbg, Some(conf.thread_pool.size)));  
    let mut dso_angles: Vec<_> = (-120..=120).map(|v| (v as f64) * 0.5 as f64).collect();
    dso_angles.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    dso_angles.dedup();    
    let mut model_cached = model_cached::ModelCached::new(
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
    let mut compartments_max = [ 
        ("214", 1.1, 125.218),
        ("228", 8.55, 211.936),
        ("229", 3.063, 10.951),
        ("215", 1.1, 106.55),
        ("201", 9.65, 166.847),
        ("217", 1.1, 126.677),
        ("202", 6.313, 133.407),
        ("216", 1.1, 106.55),
        ("212", 1.1, 82.462),
        ("206", 1.1, 69.678),
        ("400", 3.337, 6.112),
        ("602", 3.75, 36.321),
        ("401", 1.6, 5.45),
        ("207", 1.1, 234.358),
        ("213", 1.1, 125.218),
        ("991", 1.5, 135.1),
        ("205", 1.1, 221.692),
        ("211", 1.1, 82.464),
        ("403", 3.337, 6.112),
        ("601", 3.75, 36.317),
        ("402", 3.337, 35.146),
        ("303", 5.213, 113.389),
        ("302", 3.337, 17.928),
        ("504", 1.65, 4.076),
        ("706", 1.6, 6.655),
        ("704", 1.6, 10.106),
        ("301", 3.337, 20.706),
        ("705", 1.6, 6.655),
        ("701", 1.687, 3.172),
        ("503", 1.65, 4.188),
        ("305", 8.05, 81.824),
        ("304", 5.213, 113.389),
        ("502", 1, 6.676),
        ("700", 1.687, 2.959),
        ("702", 2.4, 7.807),
        ("306", 8.05, 104.765),
        ("307", 1.1, 11.888),
        ("501", 3.337, 19.034),
        ("703", 1.6, 4.515),
        ("308", 3.079, 7.806),
        ("221", 8.55, 160.421),
        ("208", 1.1, 126.681),
        ("222", 8.55, 160.421),
        ("223", 8.55, 380.631),
        ("227", 8.55, 211.936),
        ("226", 8.55, 259.222),
        ("232", 2.4, 54.66),
        ("218", 1.1, 126.687),
        ("224", 8.55, 380.631),
        ("230", 3.063, 10.951),
        ("231", 2.4, 54.66),
        ("225", 8.55, 259.222),
        ("1001", 10.8, 6298.05),
        ("1002", 10.8, 7458.75),
        ("H101", 10.8, 1657),
        ("P101", 10.8, 94.35),
        ("H102", 10.8, 1137),
        ("P102", 10.8, 94.35),
        ("H103", 10.8, 472),
        ("P103", 10.8, 94.35),
        ("H104", 10.8, 2560.3),
        ("SP101", 10.8, 94.35),
        ("SP102", 10.8, 94.35),
        ("H201", 10.8, 1819),
        ("P201", 10.8, 94.35),
        ("H202", 10.8, 450),
        ("P202", 10.8, 94.35),
        ("H203", 10.8, 1680),
        ("P203", 10.8, 94.35),
        ("H204", 10.8, 1378),
        ("P204", 10.8, 94.35),
        ("H205", 10.8, 752),
        ("P205", 10.8, 94.35),
        ("H206", 10.8, 719.3),
        ("SP201", 10.8, 94.35),
        ("SP202", 10.8, 94.35),
        ("1001_104_156", 10.8, 6298.05),
        ("1002_28_99", 10.8, 7458.75),       
    ];  
    let compartments_max: HashMap<String, (Option<f64>, f64)> = compartments_max.into_iter()
        .map(|(code, level, volume)| (code.to_owned(), (Some(level), volume))).collect();
    model_cached.reload_shapes().unwrap();
    model_cached.rebuild_compartments(&bounds, compartments_max).unwrap();
 //   model_cached.rebuild_hull(&bounds).unwrap();
 //   model_cached.rebuild_windage(&bounds).unwrap();
  //  let res = model_cached.init();                     dbg!(&res);
  //  let res = model_cached.init_bounded(&bounds);      dbg!(&res);
    Ok(())
}