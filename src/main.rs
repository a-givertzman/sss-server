mod algorithm;
mod app;
mod infrostructure;
mod kernel;
mod conf;
mod ship_model;
#[cfg(test)]
mod tests;
use algorithm::{
    context::context::Context,
    initial::{initial::Initial, initial_ctx::InitialCtx},
};
//
use api_tools::debug::dbg_id::DbgId;
use app::app::App;
use conf::conf::Conf;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use infrostructure::{api::client::api_client::ApiClient, query::restart_eval::RestartEvalQuery};
use kernel::{
    eval::Eval, run::Run,
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
    let conf = "./config.yaml";
    let conf = Conf::new(&dbg, conf);
    let _result = Initial::new(
        dbg,
        ApiClient::new(conf.api.database.clone(), conf.api.host.clone(), conf.api.port.clone()),
        Context::new(
            InitialCtx::default(),
        ),
    )
    .eval(RestartEvalQuery { ship_id: 0 })
    .await;
    Ok(())
}
