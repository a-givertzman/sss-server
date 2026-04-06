use crate::{
    algorithm::{
        context::context_access::ContextRead,
        eval::{
            Zg,
            import_model::{
                convert_surface_outer_to_trimesh::{
                    convert_surface_outer_to_trimesh_ctx::ConvertSurfaceOuterToTrimeshCtx, 
                    convert_surface_outer_to_trimesh_eval::ConvertSurfaceOuterToTrimeshEval
                }, 
                import_model_initial_points::import_model_initial_points_eval::ImportModelInitialPointsEval
            },
        },
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::{Context, InitialCtx},
};
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use parry3d_f64::shape::TriMesh;
use sal_core::error::Error;
use std::io::Write;
use std::path::PathBuf;
#[cfg(test)]
use std::{sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;
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
fn convert_surface_outer_to_trimesh() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("Starting convert_to_trimesh test");
    let test_duration = TestDuration::new("ConvertToTrimesh", Duration::from_secs(6000));
    test_duration.run().unwrap();
    let path_3d_model = "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\sophia";
    let mut initial_data = InitialCtx::new(0, "Unit-test");
    initial_data.path_3d_model = path_3d_model.to_string();
    let ctx = MocEval {
        ctx: Context::new(initial_data),
    };
    let result = ConvertSurfaceOuterToTrimeshEval::new("Test", ImportModelInitialPointsEval::new("Test", ctx))
        .eval(Zg::empty());
    match result {
        Ok(ctx) => {
            let result: ConvertSurfaceOuterToTrimeshCtx = ctx.read();
            match result.result {
                Some(mesh) => {
                    if mesh.vertices().len() > 0 {
                        let path = PathBuf::from(format!(
                            "src\\tests\\unit\\algorithm\\dialog_static\\output_files\\mesh.stl"
                        ));
                        if let Err(e) = write_stl(&path, &mesh) {
                            log::error!("Failed to write mesh {}", e);
                        }
                    }
                }
                None => log::warn!("Error to create mesh"),
            }
        }
        Err(err) => {
            log::error!("convert_to_trimesh failed with error: {:#?}", err);
            panic!("error: {:#?}", err);
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