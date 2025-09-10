use debugging::session::debug_session::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use testing::stuff::max_test_duration::TestDuration;
use std::collections::HashMap;
use std::{fs, time::Duration};
use crate::algorithm::entities::{Moment, Position, model_cached::{FloatingPositionResult, model_cached}};
///
#[test]
fn model_cached_sofia() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    let dbg = Dbg::new("test models", "model_cached_sofia");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let dbg = Dbg::new("ModelCached", "model_cached_sofia");
    let cache_dir = "src/algorithm/entities/model_cached/test/sofia".into();
    let model_dir = "".into();
    let model_center_coord = Position::new(65.250, 0., 0.);
    let thread_pool = ThreadPool::new(&dbg, Some(30));
    let mut model = model_cached::ModelCached::new(
        &dbg,
        crate::model_cached::ModelCachedConf {
            model_dir,
            cache_dir,
            model_scale: 1000.,
            model_center_coord,
            heel_steps: vec![
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., 0., 2., 5.,
                10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., 0., 1., 2., 3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            ship_length_lbp: 130.5,
            draught_min: 2.,
            draught_max: 14.,
            hull_draught_step: 1.,
            compartment_qnt_steps: 3,
            compartment_data: HashMap::new(),
        },
        thread_pool.scheduler(),
    )
    .unwrap();
    let mut result = |mass: f64, x: f64, y: f64, z: f64| {
        model.floating_position(model_cached::FloatingPositionQuery {
            water_density: 1.025,
            mass_const: mass,
            moment_const: Moment::from_pos(Position::new(x - model_center_coord.x(), y, z), mass),
            bulk: vec![],
            liquid: vec![],
            grain_bulkhead: Vec::new(),
            damaged_compartment: Vec::new(),
            epsilon: 0.0001,
        })
    };
    let data = [
        [14194.5, 63.371, -0.001, 6.605],
        [13163.9, 63.933, 0., 6.212],
        [13987., 65.231, 0., 4.99],
        [14135.3, 64.898, 0., 4.882],
        [13238.467, 65.409, 0., 6.463],
        [10960.742, 66.471, 0.001, 5.810],
        [7212.705, 66.404, 0., 5.391],
        [10000., 63.371, -0.2, 6.],
        [10000., 63.371, 0.4, 6.],
        [10000., 70., 0., 6.],
        [10000., 55., 0., 6.],
        [10000., 70., 0.1, 6.],
    ];
    let target = [
        FloatingPositionResult { heel:-0.167847, trim:-0.644409, draught_mid:8.034419, precision:0.00008578281819884135, volume:13848.29268292683, waterline_x:61.402336, waterline_y:-0.002159 },
        FloatingPositionResult { heel:-0.000244, trim:-0.520081, draught_mid:7.518466, precision:0.000021223184292953985, volume:12842.829268292684, waterline_x:61.641918, waterline_y:0.000001 },
        FloatingPositionResult { heel:0.000061, trim:-0.054871, draught_mid:7.968349, precision:0.00008890952457268238, volume:13645.853658536587, waterline_x:62.247274, waterline_y:-0.000010 },
        FloatingPositionResult { heel:0.000122, trim:-0.152039, draught_mid:8.036577, precision:0.00007848778444678247, volume:13790.536585365855, waterline_x:62.113668, waterline_y:-0.000009 },
        FloatingPositionResult { heel:0.000000, trim:-0.055664, draught_mid:7.586173, precision:0.00006624198264533723, volume:12915.577560975611, waterline_x:62.362198, waterline_y:0.000002 },
        FloatingPositionResult { heel:0.075684, trim:0.092224, draught_mid:6.406001, precision:0.00002213289278580065, volume:10693.406829268293, waterline_x:63.622290, waterline_y:0.001176 },
        FloatingPositionResult { heel:0.000488, trim:-0.092621, draught_mid:4.338902, precision:0.00001342384605676844, volume:7036.785365853659, waterline_x:66.104927, waterline_y:0.000006 },
        FloatingPositionResult { heel:-14.927734, trim:-0.746460, draught_mid:5.832851, precision:0.00008202327164830804, volume:9756.097560975611, waterline_x:62.783855, waterline_y:-0.210898 },
        FloatingPositionResult { heel:22.873047, trim:-0.661499, draught_mid:5.793020, precision:0.00007714843506647723, volume:9756.097560975611, waterline_x:63.330474, waterline_y:0.296447 },
        FloatingPositionResult { heel:-0.000488, trim:1.061768, draught_mid:5.891354, precision:0.00007275398315748077, volume:9756.097560975611, waterline_x:66.260258, waterline_y:-0.000006 },
        FloatingPositionResult { heel:-0.000244, trim:-2.878296, draught_mid:5.713602, precision:0.000035100627274754647, volume:9756.097560975611, waterline_x:59.995395, waterline_y:0.000006 },
        FloatingPositionResult { heel:9.984375, trim:1.052307, draught_mid:5.878511, precision:0.00006799904334075722, volume:9756.097560975611, waterline_x:66.288867, waterline_y:0.145717 },
    ];
    let to_array = |src: &FloatingPositionResult| vec![src.heel, src.trim, src.draught_mid, src.volume, src.waterline_x, src.waterline_y, src.precision];
    let (epsilon_p, epsilon_abs) = (0.1, 0.1);
    let compare = |result: Vec<f64>, target: Vec<f64>| {
        for (r, t) in result.iter().zip(target.iter()) {
            let delta = (r - t).abs();
            assert!(
                delta < epsilon_p * (r.abs().max(t.abs())) || delta < epsilon_abs,
                "\nresult: {:?}\ntarget: {:?}",
                result,
                target
            );
        }
    };
    for ( [m, x, y, z], target ) in data.into_iter().zip(target.iter()) {
        let target = to_array(&target);
        let result = to_array(&result(m, x, y, z).unwrap());  
        compare(result, target);  
    }
    test_duration.exit();
}
