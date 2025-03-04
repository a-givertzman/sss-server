#[cfg(test)]
mod lifting_speed {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use futures::future::BoxFuture;
    use std::{
        sync::Once,
        time::Duration,
    };
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{
        algorithm::{
            context::{context::Context, context_access::ContextRead, ctx_result::CtxResult},
            initial_ctx::initial_ctx::InitialCtx,
            lifting_speed::{lifting_speed::LiftingSpeed, lifting_speed_ctx::LiftingSpeedCtx},
        },
        kernel::{dbgid::dbgid::DbgId, eval::Eval, storage::storage::Storage, str_err::str_err::StrErr, types::eval_result::EvalResult},
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
    fn init_each() {}
    ///
    /// Testing to 'eval()' method
    #[tokio::test(flavor = "multi_thread")]
    async fn eval() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbg = DbgId("lifting_speed".into());
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data: [(i32, InitialCtx, CtxResult<f64, StrErr>); 8] = [
            (
                1,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_1",
                ))
                .unwrap(),
                CtxResult::Ok(0.63),
            ),
            (
                2,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_2",
                ))
                .unwrap(),
                CtxResult::Ok(0.2),
            ),
            (
                3,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_3",
                ))
                .unwrap(),
                CtxResult::Ok(0.2),
            ),
            (
                4,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_4",
                ))
                .unwrap(),
                CtxResult::Ok(0.315),
            ),
            (
                5,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_5",
                ))
                .unwrap(),
                CtxResult::Ok(0.0),
            ),
            (
                6,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_6",
                ))
                .unwrap(),
                CtxResult::Ok(0.63),
            ),
            (
                7,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_7",
                ))
                .unwrap(),
                CtxResult::Ok(0.2),
            ),
            (
                8,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_8",
                ))
                .unwrap(),
                CtxResult::Ok(0.315),
            ),
        ];
        for (step, initial, target) in test_data {
            let ctx = MocEval {
                ctx: Context::new(initial),
            };
            let result = LiftingSpeed::new(ctx).eval(()).await;
            match (&result, &target) {
                (CtxResult::Ok(result), CtxResult::Ok(target)) => {
                    let result = ContextRead::<LiftingSpeedCtx>::read(result)
                        .result;
                    assert!(
                        result == *target,
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    )
                }
                (CtxResult::Err(_), CtxResult::Err(_)) => {},
                (CtxResult::None, CtxResult::None) => {},
                _ => panic!("step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
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
    impl Eval<(), EvalResult> for MocEval {
        fn eval(&mut self, _: ()) -> BoxFuture<'_, EvalResult> {
            Box::pin(async {
                CtxResult::Ok(self.ctx.clone())
            })
        }
    }
}
