#[cfg(test)]

mod load_hand_device_mass {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        algorithm::{
            bearing_filter::bearing_filter_ctx::BearingFilterCtx, context::{context::Context, context_access::ContextRead, ctx_result::CtxResult}, dynamic_coefficient::dynamic_coefficient::DynamicCoefficient, hook_filter::{hook_filter::HookFilter, hook_filter_ctx::HookFilterCtx}, initial::Initial, initial_ctx::initial_ctx::InitialCtx, lifting_speed::lifting_speed::LiftingSpeed, load_hand_device_mass::{load_hand_device_mass::LoadHandDeviceMass, load_hand_device_mass_ctx::LoadHandDeviceMassCtx}, select_betta_phi::select_betta_phi::SelectBettaPhi
        },
        infrostructure::client::{choose_user_bearing::ChooseUserBearingQuery, choose_user_hook::ChooseUserHookQuery, query::Query},
        kernel::{eval::Eval, mok_user_reply::mok_user_reply::MokUserReply, request::Request, storage::storage::Storage, sync::{link::Link, switch::Switch}, user_setup::{user_bearing::UserBearing, user_hook::UserHook}}
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
    /// Testing 'eval'
    #[tokio::test(flavor = "multi_thread")]
    async fn eval() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let dbg = "load_hand_device_mass";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                r#"./src/tests/unit/kernel/storage/cache/test_2"#,
                LoadHandDeviceMassCtx { 
                    total_mass: 50.0,
                    net_weight: 8.0 
                },
            ),
            (
                2,
                r#"./src/tests/unit/kernel/storage/cache/test_3"#,
                LoadHandDeviceMassCtx { 
                    total_mass: 52.0,
                    net_weight: 18.0 
                },
            )
        ];
        let (switch, remote) = Switch::split(dbg);
        let switch_handle = switch.run().await.unwrap();
        let mut mok_user_reply = MokUserReply::new(dbg, remote);
        let mok_user_reply_handle = mok_user_reply.run().await.unwrap();
        for (step, cache_path, target) in test_data {
            let result = LoadHandDeviceMass::new(
                UserBearing::new(
                    Request::new(
                        switch.link().await,
                        async |variants: BearingFilterCtx, link: Link| {
                            let query = Query::ChooseUserBearing(ChooseUserBearingQuery::new(variants.result.clone()));
                            (link.req(query).await.expect("{}.req | Error to send request"), link)
                        }
                    ),
                    UserHook::new(
                        Request::new(
                            switch.link().await,
                            async |variants: HookFilterCtx, link: Link| {
                                let query = Query::ChooseUserHook(ChooseUserHookQuery::test(variants.result.clone()));
                                (link.req(query).await.expect("{}.req | Error to send request"), link)
                            }
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
                ),
            )
            .eval(())
            .await;
            match result {
                CtxResult::Ok(result) => {
                    let result = ContextRead::<LoadHandDeviceMassCtx>::read(&result)
                        .clone();
                    assert!(
                        result == target,
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                CtxResult::Err(err) => panic!("step {} \nerror: {:#?}", step, err),
                CtxResult::None => panic!("step {} \nerror: `UserHook` returns None", step),
            }
        }
        switch.exit();
        mok_user_reply.exit();
        mok_user_reply_handle.await.unwrap();
        switch_handle.join_all().await;
        test_duration.exit();
    }
}
