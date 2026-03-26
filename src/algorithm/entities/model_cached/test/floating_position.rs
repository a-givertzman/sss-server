use crate::algorithm::entities::{
    Bounds, Moment, Position, model_cached::{FloatingPositionResult, model_cached}
};
use debugging::session::debug_session::*;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{self, ThreadPool};
use std::collections::HashMap;
use std::{fs, time::Duration};
use testing::stuff::max_test_duration::TestDuration;

static PHYSICAL_FRAMES: [f64; 196] = [
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
///
#[test]
fn floating_position_sofia() {
    DebugSession::new().filter(LogLevel::Info).init();
    let dbg = Dbg::new("test model_cached", "floating_position_sofia");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(3000));
    test_duration.run().unwrap();
    let cache_dir = "src/algorithm/entities/model_cached/test/sofia".into();
    let model_dir = "".into();
    let model_center_coord = Position::new(65.250, 0., 0.);    
    let bounds = Bounds::from_array(&PHYSICAL_FRAMES, model_center_coord.x()).unwrap();
    let thread_pool = ThreadPool::new(&dbg, Some(20));
    let mut model = model_cached::ModelCached::new(
        &dbg,
        crate::model_cached::ModelCachedConf {
            model_dir,
            cache_dir,
            model_scale: 1000.,
            model_center_coord,
            hull_heel_steps: vec![
                -60., -50., -45., -40., -35., -30., -25., -20., -15., -10., -5., -2., 0., 2., 5.,
                10., 15., 20., 25., 30., 35., 40., 45., 50., 60.,
            ],
            hull_trim_steps: vec![
                -40., -30., -25., -20., -15., -12.5, -10., -7.5, -5., -3., -2., -1., 0., 1., 2.,
                3., 5., 7.5, 10., 12.5, 20., 25., 30., 40.,
            ],
            compartment_heel_steps: vec![
                -60., -30., -10., -5., 0., 5., 10., 30., 60.,
            ],
            compartment_trim_steps: vec![
                -40., -20., -10., -5., 0., 5., 10., 20., 40.,
            ],
            ship_length_lbp: 130.5,
            hull_draught_min: 2.,
            hull_draught_max: 14.,
            hull_draught_step: 1.,
            bounds_level_step: 1.,
            compartment_level_step: 1.
        },
        thread_pool.into(),
    )
    .unwrap();
    let compartments_volume_max = vec![
        ("201", 166.847),
        ("202", 133.407),
        ("205", 221.692),
        ("206",  69.678),
        ("207", 234.358),
        ("208", 126.681),
        ("211",  82.464),
        ("212",  82.462),
        ("213", 125.218),
        ("214", 125.218),
        ("215",  106.55),
        ("216",  106.55),
        ("217", 126.677),
        ("218", 126.687),
        ("221", 160.421),
        ("222", 160.421),
        ("223", 380.631),
        ("224", 380.631),
        ("225", 259.222),
        ("226", 259.222),
        ("227", 211.936),
        ("228", 211.936),
        ("229",  10.951),
        ("230",  10.951),
        ("231",   54.66),
        ("232",   54.66),
        ("301",  20.706),
        ("302",  17.928),
        ("303", 113.389),
        ("304", 113.389),
        ("305",  81.824),
        ("306", 104.765),
        ("307",  11.888),
        ("308",   7.806),
        ("400",   6.112),
        ("401",    5.45),
        ("402",  35.146),
        ("403",   6.112),
        ("501",  19.034),
        ("502",   6.676),
        ("503",   4.188),
        ("504",   4.076),
        ("601",  36.317),
        ("602",  36.321),
        ("700",   2.959),
        ("701",   3.172),
        ("702",   7.807),
        ("703",   4.515),
        ("704",  10.106),
        ("705",   6.655),
        ("706",   6.655),
        ("991",   135.1),
        ("11R.1", 227.2908),
        ("00-0", 20000.),
        ("1001", 6298.05),
        ("1002", 7458.75),
        ("H101", 1657.),
        ("P101", 94.35),
        ("H102", 1137.),
        ("P102", 94.35),
        ("H103", 472.),
        ("P103", 94.35),
        ("H104", 2749.),
        ("H201", 1819.),
        ("P201", 94.35),
        ("H202", 450.),
        ("P202", 94.35),
        ("H203", 1680.),
        ("P203", 94.35),
        ("H204", 1378.),
        ("P204", 94.35),
        ("H205", 752.),
        ("P205", 94.35),
        ("H206", 908.),
    ].into_iter().map(|(s, v)| (s.to_owned(), v)).collect();
    model.init(compartments_volume_max).unwrap();
    model.init_bounded(&bounds).unwrap();
    let result = |mass: f64, x: f64, y: f64, z: f64| {
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
        FloatingPositionResult {
            heel: -0.167847,
            trim: -0.644409,
            draught_mid: 8.034419,
            precision: 0.000086,
            displacement: 13848.292683,
            displacement_center: Position::new(63.344311, -0.007974, 4.224538),
            area_wl: 1940.393929,
            area_wl_center: Position::new(61.402336, -0.002159, 8.083546),
            length_wl: 134.317037,
            breadth_wl: 15.870130,
            rad_long: 178.946683,
            rad_trans: 2.730476,
        },
        FloatingPositionResult {
            heel: -0.000244,
            trim: -0.520081,
            draught_mid: 7.518466,
            precision: 0.000021,
            displacement: 12842.829268,
            displacement_center: Position::new(63.912377, -0.000011, 3.937704),
            area_wl: 1916.953198,
            area_wl_center: Position::new(61.641918, 0.000001, 7.558170),
            length_wl: 133.910390,
            breadth_wl: 15.870070,
            rad_long: 186.664918,
            rad_trans: 2.883463,
        },
        FloatingPositionResult {
            heel: 0.000061,
            trim: -0.054871,
            draught_mid: 7.968349,
            precision: 0.000089,
            displacement: 13645.853659,
            displacement_center: Position::new(65.230286, 0.000001, 4.151731),
            area_wl: 1939.707334,
            area_wl_center: Position::new(62.247274, -0.000010, 7.972640),
            length_wl: 134.264936,
            breadth_wl: 15.870101,
            rad_long: 181.450497,
            rad_trans: 2.757551,
        },
        FloatingPositionResult {
            heel: 0.000122,
            trim: -0.152039,
            draught_mid: 8.036577,
            precision: 0.000078,
            displacement: 13790.536585,
            displacement_center: Position::new(64.896098, 0.000005, 4.194825),
            area_wl: 1942.883280,
            area_wl_center: Position::new(62.113668, -0.000009, 8.048127),
            length_wl: 134.318740,
            breadth_wl: 15.870132,
            rad_long: 180.367033,
            rad_trans: 2.737663,
        },
        FloatingPositionResult {
            heel: 0.000000,
            trim: -0.055664,
            draught_mid: 7.586173,
            precision: 0.000066,
            displacement: 12915.577561,
            displacement_center: Position::new(65.406488, -0.000004, 3.945087),
            area_wl: 1915.735190,
            area_wl_center: Position::new(62.362198, 0.000002, 7.590439),
            length_wl: 133.963750,
            breadth_wl: 15.870075,
            rad_long: 185.384605,
            rad_trans: 2.860756,
        },
        FloatingPositionResult {
            heel: 0.075684,
            trim: 0.092224,
            draught_mid: 6.406001,
            precision: 0.000022,
            displacement: 10693.406829,
            displacement_center: Position::new(66.475046, 0.004304, 3.310288),
            area_wl: 1830.817511,
            area_wl_center: Position::new(63.622290, 0.001176, 6.406889),
            length_wl: 132.373264,
            breadth_wl: 15.870014,
            rad_long: 197.084339,
            rad_trans: 3.256428,
        },
        FloatingPositionResult {
            heel: 0.000488,
            trim: -0.092621,
            draught_mid: 4.338902,
            precision: 0.000013,
            displacement: 7036.785366,
            displacement_center: Position::new(66.398880, 0.000030, 2.231940),
            area_wl: 1720.672709,
            area_wl_center: Position::new(66.104927, 0.000006, 4.339615),
            length_wl: 130.073767,
            breadth_wl: 15.870000,
            rad_long: 252.430510,
            rad_trans: 4.567832,
        },
        FloatingPositionResult {
            heel: -14.927734,
            trim: -0.746460,
            draught_mid: 5.832851,
            precision: 0.000082,
            displacement: 9756.097561,
            displacement_center: Position::new(63.333039, -0.948948, 3.190758),
            area_wl: 1902.977722,
            area_wl_center: Position::new(62.783855, -0.210898, 5.928682),
            length_wl: 131.311735,
            breadth_wl: 15.870000,
            rad_long: 229.143379,
            rad_trans: 3.957635,
        },
        FloatingPositionResult {
            heel: 22.873047,
            trim: -0.661499,
            draught_mid: 5.793020,
            precision: 0.000077,
            displacement: 9756.097561,
            displacement_center: Position::new(63.338184, 1.502192, 3.387351),
            area_wl: 1986.204712,
            area_wl_center: Position::new(63.330474, 0.296447, 5.949460),
            length_wl: 131.242587,
            breadth_wl: 15.870000,
            rad_long: 239.127878,
            rad_trans: 4.536579,
        },
        FloatingPositionResult {
            heel: -0.000488,
            trim: 1.061768,
            draught_mid: 5.891354,
            precision: 0.000073,
            displacement: 9756.097561,
            displacement_center: Position::new(70.054224, -0.000024, 3.070354),
            area_wl: 1761.805450,
            area_wl_center: Position::new(66.260258, -0.000006, 5.911855),
            length_wl: 131.413297,
            breadth_wl: 15.870000,
            rad_long: 192.116427,
            rad_trans: 3.436658,
        },
        FloatingPositionResult {
            heel: -0.000244,
            trim: -2.878296,
            draught_mid: 5.713602,
            precision: 0.000035,
            displacement: 9756.097561,
            displacement_center: Position::new(54.866121, -0.000013, 3.336531),
            area_wl: 1901.151435,
            area_wl_center: Position::new(59.995395, 0.000006, 5.979971),
            length_wl: 131.104716,
            breadth_wl: 15.870000,
            rad_long: 242.452045,
            rad_trans: 3.722546,
        },
        FloatingPositionResult {
            heel: 9.984375,
            trim: 1.052307,
            draught_mid: 5.878511,
            precision: 0.000068,
            displacement: 9756.097561,
            displacement_center: Position::new(70.053662, 0.606313, 3.123589),
            area_wl: 1793.849775,
            area_wl_center: Position::new(66.288867, 0.145717, 5.925163),
            length_wl: 131.391002,
            breadth_wl: 15.870000,
            rad_long: 196.954554,
            rad_trans: 3.615159,
        },
    ];
    let to_array = |src: &FloatingPositionResult| {
        vec![
            src.heel,
            src.trim,
            src.draught_mid,
            src.displacement,
            src.precision,
            src.displacement,
            src.displacement_center.x(),
            src.displacement_center.y(),
            src.displacement_center.z(),
            src.area_wl,
            src.area_wl_center.x(),
            src.area_wl_center.y(),
            src.area_wl_center.z(),
            src.length_wl,
            src.breadth_wl,
            src.rad_long,
            src.rad_trans,
        ]
    };
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
    for ([m, x, y, z], target) in data.into_iter().zip(target.iter()) {
        let target = to_array(&target);
        let result = to_array(&result(m, x, y, z).unwrap());
        compare(result, target);
    }
    test_duration.exit();
}
