mod algorithm;
mod app;
mod infrostructure;
mod kernel;
#[cfg(test)]
mod tests;
use algorithm::{
    initial::Initial, initial_ctx::initial_ctx::InitialCtx,
    bearing_filter::bearing_filter_ctx::BearingFilterCtx, context::context::Context,
    dynamic_coefficient::dynamic_coefficient::DynamicCoefficient, hoisting_tackle::hoisting_tackle::HoistingTackle,
    hook_filter::{hook_filter::HookFilter,hook_filter_ctx::HookFilterCtx},
    lifting_speed::lifting_speed::LiftingSpeed, load_hand_device_mass::load_hand_device_mass::LoadHandDeviceMass,
    rope_count::rope_count::RopeCount, rope_effort::rope_effort::RopeEffort, select_betta_phi::select_betta_phi::SelectBettaPhi,
};
//
use api_tools::debug::dbg_id::DbgId;
use app::app::App;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use infrostructure::client::{
    change_hoisting_tackle::ChangeHoistingTackleQuery, choose_user_bearing::ChooseUserBearingQuery, choose_user_hook::ChooseUserHookQuery, query::Query
};
use kernel::{
    eval::Eval, mok_user_reply::mok_user_reply::MokUserReply, request::Request, run::Run,
    storage::storage::Storage, sync::{link::Link, switch::Switch},
    user_setup::{user_bearing::UserBearing, user_hook::UserHook},
};
///
/// Application entry point
// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// #[tokio::main]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    let dbg = DbgId("main".into());
    let path = "config.yaml";
    let mut app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    }
    let cache_path = "./src/tests/unit/kernel/storage/cache/test_2";
    let (switch, remote) = Switch::split(&dbg);
    let switch_handle = switch.run().await.unwrap();
    let mut mok_user_reply = MokUserReply::new(&dbg, remote);
    let mok_user_reply_handle = mok_user_reply.run().await.unwrap();
    let _result = HoistingTackle::new(
        Request::new(
            switch.link().await,
            async |_: (), link: Link| {
                let query = Query::ChangeHoistingTackle(ChangeHoistingTackleQuery::new());
                (link.req(query).await.expect("{}.req | Error to send request"), link)
            }
        ),
        RopeCount::new(
            RopeEffort::new(
                LoadHandDeviceMass::new(
                    UserBearing::new(
                        Request::new(
                            switch.link().await,
                            async |variants: BearingFilterCtx, link: Link| {
                                let query = Query::ChooseUserBearing(ChooseUserBearingQuery::new(variants.result.clone()));
                                (link.req(query).await.expect("{}.req | Error to send request"), link)
                            },
                        ),
                        UserHook::new(
                            Request::new(
                                switch.link().await,
                                async |variants: HookFilterCtx, link: Link| {
                                    let query = Query::ChooseUserHook(ChooseUserHookQuery::test(variants.result.clone()));
                                    (link.req(query).await.expect("{}.req | Error to send request"), link)
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
                    ),
                ),
            ),
        ),
    )
    .eval(())
    .await;
    switch.exit();
    mok_user_reply.exit();
    switch_handle.join_all().await;
    mok_user_reply_handle.await.unwrap();
    Ok(())
}
