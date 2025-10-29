#[cfg(test)]
use std::{
    sync::Once, 
    time::Duration
};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{
    DebugSession, 
    LogLevel, 
    Backtrace
};
use crate::{
    algorithm::{
        context::context_access::ContextRead, 
        eval::{
            import_3d_model_ctx::Import3DModelCtx, 
            import_3d_model_eval::Import3DModelEval, 
            Zg
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
/// returns:
///  - ...
fn init_each() -> () {}
///
/// Testing [import_3d_model_eval
/// ]
#[test]
fn import_3d_model() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "Import3DModel";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\vessel_surface_APK_2023"
        ),
        (
            2,
            "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\vessel_surface_APK_2023"
        ),
    ];
    for (step, path_3d_model) in test_data.iter() {
        let mut initial_data = InitialCtx::new(
            0,
            "Unit-test",
        );
        initial_data.path_3d_model = path_3d_model.to_string();
        let ctx = MocEval {
            ctx: Context::new(
                initial_data
            ),
        };
        let result = Import3DModelEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<Import3DModelCtx>::read(&ctx).clone();
            },
            Err(err) => panic!("step {} \nerror: {:#?}", step, err),

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
