#[cfg(test)]
mod rope_effort {
    use std::{sync::Once, time::Duration};
    use futures::future::BoxFuture;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{algorithm::{context::{context::Context, context_access::ContextRead, ctx_result::CtxResult}, initial_ctx::initial_ctx::InitialCtx, rope_effort::{rope_effort::RopeEffort, rope_effort_ctx::RopeEffortCtx}}, kernel::{eval::Eval, storage::storage::Storage, types::eval_result::EvalResult}};
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
    /// Testing 'eval'
    #[tokio::test(flavor = "multi_thread")]
    async fn eval() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "rope_effort";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_1",
                ))
                .unwrap(),
                90.0
            ),
            (
                2,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_2",
                ))
                .unwrap(),
                30.0
            ),
            (
                3,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_3",
                ))
                .unwrap(),
                50.0
            )
        ];
        for (step,initial,target) in test_data {
            let ctx = MocEval {
                ctx: Context::new(initial),
            };
            let result = RopeEffort::new(ctx).eval(()).await;
            match &result {
                CtxResult::Ok(result) => {
                    let result = ContextRead::<RopeEffortCtx>::read(result)
                        .result;
                    assert!(
                        result == target,
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    )
                }
                CtxResult::Err(err) => panic!("step {} \nerror: {:#?}", step, err),
                CtxResult::None => panic!("step {} \nerror: `RopeEffort` returns None", step),
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
