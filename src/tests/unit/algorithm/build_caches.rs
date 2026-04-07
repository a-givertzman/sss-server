use std::path::PathBuf;
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
    let mut dso_angles = vec![-60., -50., -40., -30., -12., 12., 30., 40., 50., 60.];
    dso_angles.append(&mut ((-11..=11).map(|v| (v as f64) * 5.).collect())); // -55, -50 .. 55
    dso_angles.append(&mut ((-8..=8).map(|v| v as f64).collect()));
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
    model_cached.reload_shapes().unwrap();
    model_cached.rebuild_hull(&bounds).unwrap();
  //  let res = model_cached.init();                     dbg!(&res);
  //  let res = model_cached.init_bounded(&bounds);      dbg!(&res);
    Ok(())
}