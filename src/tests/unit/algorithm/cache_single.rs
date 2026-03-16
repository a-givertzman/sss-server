use std::path::PathBuf;

use crate::algorithm::entities::model_cached::{self, DisplacementShape, LocalCache, Shape};
#[cfg(test)]
use crate::app::app::App;
use crate::conf::Conf;
use crate::kernel::{
    run::Run,
    types::{Arc, RwLock},
};
use debugging::session::debug_session::{DebugSession, LogLevel};
use sal_core::dbg::Dbg;
use sal_sync::thread_pool::ThreadPool;
///
/// Application entry point
#[ignore = "too slow, run only in release mode"]
#[test]
fn cache_single_sofia_201() -> Result<(), Box<dyn std::error::Error>> {
    DebugSession::new()
        //  .filter(LogLevel::Info)
        .filter(LogLevel::Debug)
        //   .filter(LogLevel::Trace)
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
    let thread_pool = Arc::new(ThreadPool::new(&dbg, Some(conf.thread_pool.size)));
    let cache_dir: PathBuf = "src/tests/unit/algorithm/cache/assets".into();
    let model_dir: PathBuf = "assets/model/sofia/compartments/201.stl".into();
    let shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
        &dbg, model_dir, None, 1000.,
    )));
    shape.write().init().unwrap();
    let mut cache = model_cached::CompartmentCache::new(
        &dbg,
        shape.clone(),
        cache_dir,
        "201".to_owned(),
        vec![
            -60., -30., -15., -10., -5., -2., 0., 2., 5., 10., 15., 30., 60.,
        ],
        vec![-40., -20., -10., -5., -2., 0., 2., 5., 10., 20., 40.],
        20,
        Arc::clone(&thread_pool),
    );
    cache.rebuild().unwrap();
    cache.calc_coeff(168.4, Some(9.65)).unwrap();
    dbg!(cache.get_level(-15.0, 3., 135., 0.000001));
    Ok(())
}
///
#[ignore = "too slow, run only in release mode"]
#[test]
fn cache_single_ARK_02() -> Result<(), Box<dyn std::error::Error>> {
    DebugSession::new()
        //  .filter(LogLevel::Info)
        .filter(LogLevel::Debug)
        //   .filter(LogLevel::Trace)
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
    let thread_pool = Arc::new(ThreadPool::new(&dbg, Some(conf.thread_pool.size)));
    let cache_dir: PathBuf = "src/tests/unit/algorithm/cache/assets".into();
    let model_dir: PathBuf = "assets/model/ARK/compartments/02.stl".into();
    let shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
        &dbg, model_dir, None, 1.,
    )));
    shape.write().init().unwrap();
    dbg!(shape.read().center());
    let mut cache = model_cached::CompartmentCache::new(
        &dbg,
        shape.clone(),
        cache_dir,
        "test_ark".to_owned(),
        vec![
            -60., -30., -15., -10., -5., -2., 0., 2., 5., 10., 15., 30., 60.,
        ],
        vec![-40., -20., -10., -5., -2., 0., 2., 5., 10., 20., 40.],
        20,
        Arc::clone(&thread_pool),
    );
    cache.init().unwrap();
  //  cache.rebuild().unwrap();
    cache.calc_coeff(58.79, Some(6.801)).unwrap();
    for i in 0..=7 {
        let res = cache.get_volume(0., 0., i as f64).unwrap();
        println!("l:{i} v:{:.3}, center:{}, ix:{:.3}, iy:{:.3}", res.volume, res.volume_center, res.inertia_trans_x, res.inertia_long_y);
    }
    Ok(())
}
///
#[test]
fn cache_single_ARK_hull() -> Result<(), Box<dyn std::error::Error>> {
    DebugSession::new()
        //  .filter(LogLevel::Info)
        .filter(LogLevel::Debug)
        //   .filter(LogLevel::Trace)
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
    let thread_pool = Arc::new(ThreadPool::new(&dbg, Some(conf.thread_pool.size)));
    let cache_dir: PathBuf = "src/tests/unit/algorithm/cache/assets".into();
    let model_dir: PathBuf = "assets/model/ARK/hull.stl".into();
    let shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
        &dbg, model_dir, None, 1.,
    )));
    shape.write().init().unwrap();
    dbg!(shape.read().center());
    let mut cache = model_cached::DisplacementCache::new(
        &dbg,
        shape.clone(),
        cache_dir,
        vec![
            -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., -1., -0.5,
            -0.2, 0., 0.2, 0.5, 1., 2., 5., 10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
        ],
        vec![-40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., -0.5, -0.2,
                0., 0.2, 0.5, 1., 2., 3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,],
        1.40,
        10.,
        0.5,
        Arc::clone(&thread_pool),
    );
    cache.rebuild().unwrap();    
    cache.init().unwrap();
    let epsilon = 0.001;
    let show = |heel, trim, volume| {
        let res = cache.get(heel, trim, volume, epsilon).unwrap();
        println!("heel:{:.3} trim:{:.3}, volume:{:.3}, draught:{:.3}, v_center:{}, l_wl:{:.3}, b_wl:{:.3}, wl_center:{}", 
        heel, trim, volume, res.draught, res.volume_center, res.length_wl, res.breadth_wl, res.area_wl_center);
    };
    show(2.,1., 5000. );
    show(-2.,-1., 5000. );
    show(2.,1., 10000. );
    show(-2.,-1., 10000. );   
    Ok(())
}

