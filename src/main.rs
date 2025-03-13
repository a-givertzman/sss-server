mod algorithm;
mod app;
mod infrostructure;
mod kernel;
mod conf;
mod ship_model;
#[cfg(test)]
mod tests;
mod prelude;

use algorithm::eval::*;
//
use api_tools::debug::dbg_id::DbgId;
use app::app::App;
use conf::conf::Conf;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use infrostructure::{api::client::api_client::ApiClient, query::restart_eval::RestartEvalQuery};
use kernel::{
    eval::Eval, run::Run,
};
use ship_model::ship_model::ShipModel;
use prelude::*;

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
    let ship_id = 0;
    let ship_model = ShipModel::new(
        &dbg,
        ship_id,
        ApiClient::new(conf.api.address.database.clone(), conf.api.address.host.clone(), conf.api.address.port.clone()),
    );
    let ship_model_handle = ship_model.run().await.unwrap();
    log::debug!("main | Calculations...");
    let _result = 
    IcingStabEval::new(
        &dbg,
        AreasStrength::new(
            &dbg,
            ship_model.link().await,
            Initial::new(
                &dbg,
                ApiClient::new(conf.api.address.database.clone(), conf.api.address.host.clone(), conf.api.address.port.clone()),
                Context::new(
                    InitialCtx::new(
                        ship_id,
                    ),
                ),
            ),
        ),
    )
    .eval(())
    .await;
    ship_model.exit();
    ship_model_handle.await.unwrap();
    Ok(())
}
