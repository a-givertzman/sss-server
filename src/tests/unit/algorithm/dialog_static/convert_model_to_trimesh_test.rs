use std::path::PathBuf;
#[cfg(test)]
use std::{
    sync::Once, 
    time::Duration
};
use parry3d_f64::shape::TriMesh;
use sal_core::error::Error;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{
    DebugSession, 
    LogLevel, 
    Backtrace
};
use std::io::Write;
use crate::{
    algorithm::{
        context::context_access::ContextRead, 
        eval::{
            Zg, import_model::{
                convert_model_to_trimesh_ctx::ConvertModelToTrimeshCtx, 
                convert_model_to_trimesh_eval::ConvertModelToTrimeshEval, 
                import_3d_model_eval::Import3DModelEval
            }, 
        }
    }, 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{
        Context, 
        InitialCtx
    }
};
///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// Write data to .stl file
pub fn write_stl(path: &PathBuf, mesh: &TriMesh) -> Result<(), Error> {
    let error = Error::new("Shape", "write_stl");
    let (result, empty_normals): (Vec<_>, Vec<_>) = mesh
        .triangles()
        .map(|t| (t.normal(), t))
        .partition(|(n, _)| n.is_some());
    if !empty_normals.is_empty() {
        return Err(error.err(format!("calculate normal error, path:{:?}", path)));
    }
    let triangles: Vec<_> = result
        .into_iter()
        .map(|(n, t)| {
            let n = n.unwrap();
            let normal = stl_io::Vector([n[0] as f32, n[1] as f32, n[2] as f32]);
            let vertices = [
                stl_io::Vector([t.a[0] as f32, t.a[1] as f32, t.a[2] as f32]),
                stl_io::Vector([t.b[0] as f32, t.b[1] as f32, t.b[2] as f32]),
                stl_io::Vector([t.c[0] as f32, t.c[1] as f32, t.c[2] as f32]),
            ];
            stl_io::Triangle { normal, vertices }
        })
        .collect();
    let mut binary_stl = Vec::<u8>::new();
    stl_io::write_stl(&mut binary_stl, triangles.iter())
        .map_err(|err| error.pass_with("stl_io::write_stl", err.to_string()))?;
    let mut buffer = std::fs::File::create(&path).map_err(|err| {
        error.pass_with(format!("File::create, path:{:?}", path), err.to_string())
    })?;
    buffer.write_all(&binary_stl).map_err(|err| {
        error.pass_with(
            format!("buffer.write_all, path:{:?}", path),
            err.to_string(),
        )
    })
}
///
/// Calculate volume of TriMesh
pub fn volume(mesh: &TriMesh) -> f64 {
    let inv_mass = parry3d_f64::shape::Shape::mass_properties(mesh, 1.).inv_mass;
    if inv_mass > 0. { 1. / inv_mass } else { 0. }
}
///
/// returns:
///  - ...
fn init_each() -> () {}
///
/// Testing [convert_to_trimesh]
#[test]
fn convert_model_to_trimesh() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("Starting convert_model_to_trimesh test");
    let test_duration = TestDuration::new("ConvertModelToTrimesh", Duration::from_secs(6000));
    test_duration.run().unwrap();
    let error_percent = 1.0;
    let test_data = [
        // (
        //    1, 
        //    "unboxes_АРК_2023",
        //    "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\unboxes_АРК_2023",
        //    12068.8268,
        // ),
        // (
        //    2, 
        //    "APK_2023",
        //    "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\APK_2023",
        //    2363.6901869983108,
        // ),
        // (
        //    3,
        //    "sophia",
        //    "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\sophia",
        //    21360.5678,
        // ),
        (
           4, 
           "katamaran",
           "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\katamaran",
           1986.66182,
        ),
    ];
    for (step, ship_name, path_3d_model, target) in test_data.iter() {
        log::debug!("Step {}: processing {}", step, path_3d_model);
        let mut initial_data = InitialCtx::new(0, "Unit-test");
        initial_data.path_3d_model = path_3d_model.to_string();
        let ctx = MocEval {
            ctx: Context::new(initial_data),
        };
        match ConvertModelToTrimeshEval::new("Test", Import3DModelEval::new("Test", ctx))
            .eval(Zg::empty()) {
            Ok(ctx) => {
                match ContextRead::<ConvertModelToTrimeshCtx>::read(&ctx).clone().surface_outer_body.clone() {
                    Some(surface_outer_body) => {
                        let mut result = 0.0;
                        let path = PathBuf::from(format!("src/tests/unit/algorithm/dialog_static/output_files/{}.stl", ship_name));
                        if let Err(e) = write_stl(&path, &surface_outer_body) {
                            log::error!("Failed to write nasal mesh {}", e);
                        }
                        result += volume(&surface_outer_body);
                        let current_error = (result - target).abs() / ((result + target) / 2.0);
                        log::debug!("Result volume: {:?}", result);
                        log::debug!("Target volume: {:?}", target);
                        assert!(current_error <= error_percent, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                    },
                    None => {
                        log::debug!("Error to calculate TriMesh from model: {}", path_3d_model);
                    },
                }
            },
            Err(err) => {
                log::error!("Step {} failed with error: {:#?}", step, err);
                panic!("step {} \nerror: {:#?}", step, err);
            },
        }
    }
    test_duration.exit();
}
///
///
#[derive(Debug, Clone)]
struct MocEval {
    pub ctx: Context,
}
//
//
impl Eval<Zg, EvalResult> for MocEval {
    fn eval(&self, _zg: Zg) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}