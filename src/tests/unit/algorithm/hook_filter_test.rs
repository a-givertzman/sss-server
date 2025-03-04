#[cfg(test)]

mod hook_filter {
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
            entities::hook::Hook,
            hook_filter::{hook_filter::HookFilter, hook_filter_ctx::HookFilterCtx},
            initial_ctx::initial_ctx::InitialCtx,
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
        let dbg = DbgId("hook_filter".into());
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_1",
                ))
                .unwrap(),
                CtxResult::Err(StrErr(format!("HookFilter.{} | No available variants of hook for specified requirements",dbg))),
            ),
            (
                2,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_2",
                ))
                .unwrap(),
                CtxResult::Ok(vec![
                    Hook {
                        gost: "GOST 18442-81".to_string(),
                        r#type: "Double".to_string(),
                        load_capacity_m13: 12.0,
                        load_capacity_m46: 11.0,
                        load_capacity_m78: 10.0,
                        shank_diameter: 55.0,
                        weight: 60.0,
                    },
                    Hook {
                        gost: "GOST 23858-79".to_string(),
                        r#type: "Forged".to_string(),
                        load_capacity_m13: 22.0,
                        load_capacity_m46: 20.0,
                        load_capacity_m78: 18.5,
                        shank_diameter: 80.0,
                        weight: 70.0,
                    },
                    Hook {
                        gost: "GOST 31272-92".to_string(),
                        r#type: "Laminated".to_string(),
                        load_capacity_m13: 17.0,
                        load_capacity_m46: 16.0,
                        load_capacity_m78: 14.0,
                        shank_diameter: 65.0,
                        weight: 80.0,
                    },
                ]),
            ),
            (
                3,
                InitialCtx::new(&mut Storage::new(
                    "./src/tests/unit/kernel/storage/cache/test_3",
                ))
                .unwrap(),
                CtxResult::Ok(vec![Hook {
                    gost: "GOST 34567-85".to_string(),
                    r#type: "Forged".to_string(),
                    load_capacity_m13: 25.0,
                    load_capacity_m46: 23.0,
                    load_capacity_m78: 21.0,
                    shank_diameter: 85.0,
                    weight: 70.0,
                }]),
            ),
        ];
        for (step, initial, target) in test_data {
            let ctx = MocEval {
                ctx: Context::new(initial),
            };
            let result = HookFilter::new(ctx).eval(()).await;
            match (&result, &target) {
                (CtxResult::Ok(result), CtxResult::Ok(target)) => {
                    let result = ContextRead::<HookFilterCtx>::read(result)
                        .result
                        .clone();
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
