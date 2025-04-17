mod algorithm;
mod app;
mod infrostructure;
mod kernel;
mod conf;
mod ship_model;
mod prelude;
#[cfg(test)]
mod tests;

use algorithm::eval::*;
//
use api_tools::debug::dbg_id::DbgId;
use app::app::App;
use conf::conf::Conf;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use infrostructure::api::client::api_client::ApiClient;
use kernel::{
    eval::Eval, run::Run,
};
use ship_model::ship_model::ShipModel;
use prelude::*;

///
/// Application entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    let dbg = DbgId("main".into());
    let path = "config.yaml";
    let mut app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    }
    let conf = "./config.yaml";
    let conf = Conf::new(&dbg, conf);
    let ship_id = 2;
    let project_id = "NULL";
    let n_parts = 200;
    let ship_model = ShipModel::new(
        &dbg,
        ship_id,
        project_id.to_owned(),
        n_parts,
        ApiClient::new(conf.api.address.database.clone(), conf.api.address.host.clone(), conf.api.address.port.clone()),
    );
    let ship_model_handle = ship_model.run().unwrap();
    log::debug!("main | Calculations...");
    let _result =     
    BalanceEval::new(
        &dbg,  
        ship_model.link(),
        LoadsEval::new(
            &dbg,        
            WettingEval::new(
                &dbg,   
                IcingEval::new(
                    &dbg,
                    StrengthAreaEval::new(
                        &dbg,
                        ship_model.link(),
                        IcingTimberEval::new(
                            &dbg,
                            IcingStabEval::new(
                                &dbg,
                                Initial::new(
                                    &dbg,
                                    ship_model.link(),
                                    ApiClient::new(conf.api.address.database.clone(), conf.api.address.host.clone(), conf.api.address.port.clone()),
                                    Context::new(
                                        InitialCtx::new(
                                            ship_id,
                                            project_id,
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
    ;
    ship_model.exit();
    ship_model_handle.join().unwrap();
    Ok(())
}
