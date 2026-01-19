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
            Zg, 
            import_tanks::{
                import_3d_tanks_ctx::Import3DTanksCtx,
                import_3d_tanks_eval::Import3DTanksEval,
            }
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
/// Testing [import_3d_tanks_eval]
/// 
#[test]
fn import_3d_tanks() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "Import3DTanks";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            "src\\tests\\unit\\algorithm\\dialog_static\\test_files\\tanks_test_1"
        ),
    ];
    for (step, path_3d_tanks) in test_data.iter() {
        let mut initial_data = InitialCtx::new(
            0,
            "Unit-test",
        );
        initial_data.path_3d_tanks = path_3d_tanks.to_string();
        let ctx = MocEval {
            ctx: Context::new(
                initial_data
            ),
        };
        let result = Import3DTanksEval::new("Test", ctx).eval(Zg::empty());
        match result {
            Ok(ctx) => {
                let result = ContextRead::<Import3DTanksCtx>::read(&ctx).clone();
                println!("{:?}", result.compartment_corner_points);
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
