#[cfg(test)]

mod bearing_filter {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        algorithm::{
            bearing_filter::{bearing_filter::BearingFilter, bearing_filter_ctx::BearingFilterCtx}, context::{context::Context, context_access::ContextRead, ctx_result::CtxResult}, dynamic_coefficient::dynamic_coefficient::DynamicCoefficient, entities::bearing::Bearing, hook_filter::{hook_filter::HookFilter, hook_filter_ctx::HookFilterCtx}, initial::Initial, initial_ctx::initial_ctx::InitialCtx, lifting_speed::lifting_speed::LiftingSpeed, select_betta_phi::select_betta_phi::SelectBettaPhi
        },
        infrostructure::client::{choose_user_hook::{ChooseUserHookQuery, ChooseUserHookReply}, query::Query},
        kernel::{eval::Eval, sync::link::Link, mok_user_reply::mok_user_reply::MokUserReply, request::Request, storage::storage::Storage, sync::switch::Switch, user_setup::user_hook::UserHook}
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
    /// Testing 'eval' method
    // #[tokio::test(flavor = "multi_thread")]
    #[tokio::test(flavor = "multi_thread")]
    async fn eval() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "bearing_filter";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                r#"./src/tests/unit/kernel/storage/cache/test_2"#,
                vec![
                    Bearing { 
                        name: "Bearing A".to_string(),
                        outer_diameter: 120.5,
                        inner_diameter: 60.2,
                        static_load_capacity: 1500.0,
                        height: 35.0 
                    }
                ],
            ),
            (
                2,
                r#"./src/tests/unit/kernel/storage/cache/test_3"#,
                vec![
                    Bearing { 
                        name: "Bearing A".to_string(),
                        outer_diameter: 120.5,
                        inner_diameter: 60.2,
                        static_load_capacity: 1500.0,
                        height: 35.0 
                    },
                    Bearing { 
                        name: "Bearing B".to_string(),
                        outer_diameter: 100.0,
                        inner_diameter: 50.0,
                        static_load_capacity: 1200.0,
                        height: 30.0 
                    },
                    Bearing { 
                        name: "Bearing C".to_string(),
                        outer_diameter: 140.0,
                        inner_diameter: 70.0,
                        static_load_capacity: 1800.0,
                        height: 40.0 
                    }
                ],
            )
        ];
        let (switch, remote) = Switch::split(dbg);
        let switch_handle = switch.run().await.unwrap();
        let mut mok_user_reply = MokUserReply::new(dbg, remote);
        let mok_user_reply_handle = mok_user_reply.run().await.unwrap();
        for (step, cache_path, target) in test_data {
            let result = BearingFilter::new(
                UserHook::new(
                    Request::new(
                        switch.link().await,
                        async |variants: HookFilterCtx, link: Link| {
                            let variants = variants.result.clone();
                            let query = Query::ChooseUserHook(ChooseUserHookQuery::test(variants));
                            (link.req::<ChooseUserHookReply>(query).await.expect("{}.req | Error to send request"), link)
                        },
                    ),
                    HookFilter::new(
                        DynamicCoefficient::new(
                            SelectBettaPhi::new(
                                LiftingSpeed::new(
                                    Initial::new(
                                        Context::new(
                                            InitialCtx::new(
                                                &mut Storage::new(cache_path)
                                            ).unwrap(),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            )
            .eval(())
            .await;
            match result {
                CtxResult::Ok(result) => {
                    let result = ContextRead::<BearingFilterCtx>::read(&result)
                        .result
                        .clone();
                    log::debug!("step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                    assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                }
                CtxResult::Err(err) => panic!("step {} \nerror: {:#?}", step, err),
                CtxResult::None => panic!("step {} \nerror: `UserHook` returns None", step),
            }
        }
        log::debug!("{} | Exiting...", dbg);
        switch.exit();
        mok_user_reply.exit();
        mok_user_reply_handle.await.unwrap();
        switch_handle.join_all().await;
        test_duration.exit();
    }
}
