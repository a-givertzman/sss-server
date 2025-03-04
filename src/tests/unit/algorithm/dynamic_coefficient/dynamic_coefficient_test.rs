#[cfg(test)]

mod dynamic_coefficient {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use futures::future::BoxFuture;
    use std::{
        sync::Once,
        time::Duration,
    };
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{
        algorithm::{
            context::{context::Context, context_access::{ContextRead, ContextWrite}, ctx_result::CtxResult}, dynamic_coefficient::{dynamic_coefficient::DynamicCoefficient, dynamic_coefficient_ctx::DynamicCoefficientCtx}, entities::bet_phi::BetPhi, hook_filter::hook_filter_ctx::HookFilterCtx, initial_ctx::initial_ctx::InitialCtx, lifting_speed::lifting_speed_ctx::LiftingSpeedCtx, select_betta_phi::select_betta_phi_ctx::SelectBetPhiCtx
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
        let dbg = DbgId("dynamic_coefficient".into());
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data: [(i32, Context, CtxResult<f64, StrErr>); 3] = [
            (
                1,
                {
                    let ctx = Context::new(
                            InitialCtx::new(&mut Storage::new(
                                "./src/tests/unit/kernel/storage/cache/test_1",
                            ),
                        ).unwrap(),
                    );
                    let ctx = ctx.write(LiftingSpeedCtx {
                        result: 50.0,
                    }).unwrap();
                    let ctx = ctx.write(SelectBetPhiCtx {
                        result: BetPhi {
                            bet: 5.0,
                            phi: 15.0,
                        },
                    }).unwrap();
                    let ctx = ctx.write(DynamicCoefficientCtx::default()).unwrap();
                    ctx.write(HookFilterCtx::default()).unwrap()
                },
                CtxResult::Ok(265.0),
            ),
            (
                2,
                {
                    let ctx = Context::new(
                        InitialCtx::new(&mut Storage::new(
                        "./src/tests/unit/kernel/storage/cache/test_1",
                        )).unwrap(),
                    );
                    let ctx = ctx.write(LiftingSpeedCtx {
                        result: 50.0,
                    }).unwrap();
                    let ctx = ctx.write(SelectBetPhiCtx {
                        result: BetPhi {
                            bet: 52.0,
                            phi: 16.0,
                        },
                    }).unwrap();
                    let ctx = ctx.write(DynamicCoefficientCtx::default()).unwrap();
                    ctx.write(HookFilterCtx::default()).unwrap()
                },
                CtxResult::Ok(2616.0),
            ),
            (
                3,
                {
                    let ctx = Context::new(
                        InitialCtx::new(&mut Storage::new(
                            "./src/tests/unit/kernel/storage/cache/test_1",
                        )).unwrap(),
                    );
                    let ctx = ctx.write(LiftingSpeedCtx {
                        result: 50.0,
                    }).unwrap();
                    let ctx = ctx.write(SelectBetPhiCtx {
                        result: BetPhi {
                            bet: 35.0,
                            phi: 25.0,
                    },
                    }).unwrap();
                    let ctx = ctx.write(DynamicCoefficientCtx::default()).unwrap();
                    ctx.write(HookFilterCtx::default()).unwrap()
                },
                CtxResult::Ok(1775.0),
            ),
        ];
        for (step, ctx, target) in test_data {
            let ctx = MocEval { ctx };
            let result = DynamicCoefficient::new(ctx)
                .eval(())
                .await;
            match (&result, &target) {
                (CtxResult::Ok(result), CtxResult::Ok(target)) => {
                    let result = ContextRead::<DynamicCoefficientCtx>::read(result)
                        .result;
                    assert!(
                        result == *target,
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
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
    #[derive(Debug)]
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
