use std::path::PathBuf;
#[cfg(test)]
use std::{
    sync::Once, 
    time::Duration
};
use parry3d_f64::shape::{TriMesh, TriMeshFlags};
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
            Zg, 
            convert_to_trimesh_ctx::ConvertToTrimeshCtx, 
            convert_to_trimesh_eval::ConvertToTrimeshEval, 
            import_3d_model_eval::Import3DModelEval
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
/// returns:
///  - ...
fn init_each() -> () {}
///
/// Testing [convert_to_trimesh]
#[test]
fn convert_to_trimesh() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("Starting convert_to_trimesh test");
    let test_duration = TestDuration::new("ConvertToTrimesh", Duration::from_secs(31));
    test_duration.run().unwrap();
    let test_data = [
        (1, "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\nasal_block"),
    ];
    for (step, path_3d_model) in test_data.iter() {
        log::debug!("Step {}: processing {}", step, path_3d_model);
        let mut initial_data = InitialCtx::new(0, "Unit-test");
        initial_data.path_3d_model = path_3d_model.to_string();
        let ctx = MocEval {
            ctx: Context::new(initial_data),
        };
        let result = ConvertToTrimeshEval::new("Test", Import3DModelEval::new("Test", ctx))
            .eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<ConvertToTrimeshCtx>::read(&ctx).clone();
                let mut i = 0;
                for mesh in &result.nasal_block {
                    let mut mesh_with_flags = mesh.clone();
                    let _ = mesh_with_flags.set_flags(TriMeshFlags::all());
                    if mesh_with_flags.vertices().len() > 0 {
                        let path = PathBuf::from(format!("src\\tests\\unit\\algorithm\\dialog_static\\output_files\\nasal_{}.stl", i));
                        if let Err(e) = write_stl(&path, &mesh_with_flags) {
                            log::error!("Failed to write nasal mesh {}: {}", i, e);
                        }
                        i += 1;
                    }
                }
                let mut i = 0;
                for mesh in &result.stern_block {
                    let mut mesh_with_flags = mesh.clone();
                    let _ = mesh_with_flags.set_flags(TriMeshFlags::all());
                    if mesh.vertices().len() > 0 {
                        let path = PathBuf::from(format!("src\\tests\\unit\\algorithm\\dialog_static\\output_files\\stern_{}.stl", i));
                        if let Err(e) = write_stl(&path, &mesh_with_flags) {
                            log::error!("Failed to write nasal mesh {}: {}", i, e);
                        }
                        i += 1;
                    }
                }
                let mut i = 0;
                for mesh in &result.surface_outer_body {
                    let mut mesh_with_flags = mesh.clone();
                    let _ = mesh_with_flags.set_flags(TriMeshFlags::all());
                    if mesh_with_flags.vertices().len() > 0 {
                        let path = PathBuf::from(format!("src\\tests\\unit\\algorithm\\dialog_static\\output_files\\surface_outer_{}.stl", i));
                        if let Err(e) = write_stl(&path, &mesh_with_flags) {
                            log::error!("Failed to write nasal mesh {}: {}", i, e);
                        }
                        i += 1;
                    }
                }
                let mut i = 0;
                for mesh in &result.surface_superstructure {
                    let mut mesh_with_flags = mesh.clone();
                    //let _ = mesh_with_flags.set_flags(TriMeshFlags::all());
                    if mesh_with_flags.vertices().len() > 0 {
                        let path = PathBuf::from(format!("src\\tests\\unit\\algorithm\\dialog_static\\output_files\\surface_superstructure_{}.stl", i));
                        if let Err(e) = write_stl(&path, &mesh_with_flags) {
                            log::error!("Failed to write nasal mesh {}: {}", i, e);
                        }
                        i += 1;
                    }
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
