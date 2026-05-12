use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::stability::criterion::CriterionRelationArray;
use crate::algorithm::eval::ZgCtx;
use crate::algorithm::eval::criterion::{
    CriterionData, CriterionDraughtCtx, CriterionStabilityCtx,
};
use crate::algorithm::eval::parameters::{IParameters, Parameters};
use crate::infrostructure::ApiClient;
use crate::kernel::types::Arc;
use crate::{
    algorithm::context::context_access::ContextReadRef,
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextRead, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use std::collections::HashMap;
///
/// Результаты расчета критериев
pub struct ResultCriterionEval {
    dbg: Dbg,
    api_client: Arc<ApiClient>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ResultCriterionEval {
    //
    pub fn new(
        parent: impl Into<String>,
        api_client: Arc<ApiClient>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ResultCriterionEval");
        Self {
            dbg,
            api_client,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for ResultCriterionEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let ship_id = initial.ship_id.clone();
                let project_id = initial.project_id.clone();
                let criterion_stability: CriterionStabilityCtx = ctx.read();
                let criterion_draught: CriterionDraughtCtx = ctx.read();
                let zg: ZgCtx = ctx.read();
                let parameters: &Parameters = ctx.read_ref();
                send_stability_data(
                    &self.dbg,
                    &ship_id,
                    &project_id,
                    &self.api_client,                    
                    &criterion_stability.data,
                    &zg.data,
                    &criterion_draught.data,
                )
                .map_err(|err| error.pass(err))?;
                send_parameters_data(
                    &self.dbg,
                    &ship_id,
                    &project_id,
                    &self.api_client,                    
                    parameters.clone().take_data(),
                )
                .map_err(|err| error.pass(err))?;
                Ok(ctx)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ResultCriterionEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultCriterionEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
/// Запись данных расчета остойчивости в БД
pub fn send_stability_data(
    dbg: &Dbg,
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
    criterion_data: &Vec<CriterionData>,
    zg_data: &HashMap<usize, f64>,
    criterion_draugth: &Vec<CriterionData>,
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_stability_diagram");
    log::info!("send_stability_data begin");
    log::info!("read criterion relaton begin");
    let relaton = CriterionRelationArray::parse(
        &api_client
            .fetch("SELECT id, math_relation::TEXT AS relation FROM criterion;")
            .map_err(|err| error.pass(err))?,
    )
    .map_err(|err| error.pass(err))?
    .data();
    log::info!("read criterion relaton ok");
    log::info!("send_stability_data send begin");
    let mut full_sql = "DO $$ BEGIN ".to_owned();
    full_sql += &format!("DELETE FROM criterion_values WHERE ship_id = {ship_id} AND project_id IS NOT DISTINCT FROM {project_id};");
    criterion_data.iter().for_each(|v| {
        let zg_value = if let Some(zg) = zg_data.get(&v.criterion_id) {
            zg.to_string()
        } else {
            "NULL".to_owned()
        };
        let state = match relaton.get(&(v.criterion_id as i32)).unwrap_or(&("".to_owned())).as_str() {
            "<=" => (v.result <= v.target).to_string(), 
            ">=" => (v.result >= v.target).to_string(),
            _ => "NULL".to_owned(),
        };
        full_sql += " INSERT INTO criterion_values "; 
        full_sql += &match v.error_message.clone() {
            Some(error) => format!("(ship_id, criterion_id, actual_value, limit_value, zg_value, state, error_message) VALUES ({ship_id}, {}, {}, {}, {state}, {zg_value}, '{}');", v.criterion_id, v.result, v.target, error),
            None => format!("(ship_id, criterion_id, actual_value, limit_value, state, zg_value) VALUES ({ship_id}, {}, {}, {}, {state}, {zg_value});", v.criterion_id, v.result, v.target),
        };
    });
    criterion_draugth.iter().for_each(|v| {
        let state = match relaton.get(&(v.criterion_id as i32)).unwrap_or(&("".to_owned())).as_str() {
            "<=" => (v.result <= v.target).to_string(), 
            ">=" => (v.result >= v.target).to_string(),
            _ => "NULL".to_owned(),
        };
        full_sql += " INSERT INTO criterion_values "; 
        full_sql += &match v.error_message.clone() {
            None => format!("(ship_id, project_id, criterion_id, actual_value, limit_value, state) VALUES ({ship_id}, {project_id}, {}, {}, {}, {state});", v.criterion_id, v.result, v.target),
            Some(error) => format!("(ship_id, project_id, criterion_id, actual_value, limit_value, state, error_message) VALUES ({ship_id}, {project_id}, {}, {}, {}, {state}, '{}');", v.criterion_id, v.result, v.target, error),
        };
    });
    full_sql += " END$$;";
    api_client.fetch(&full_sql).map_err(|err| error.pass(err))?;
    log::info!("send_stability_data end");
    Ok(())
}
/// Запись данных расчета остойчивости в БД
pub fn send_parameters_data(
    dbg: &Dbg,
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
    data: Vec<(usize, f64)>,
) -> Result<(), Error> {
    let error = Error::new(dbg, "send_parameters_data");
    log::info!("send_parameters_data begin");
    if data.is_empty() {
        return Err(error.err("empty data!"));
    }
    let data_list: Vec<_> =  data.into_iter().map(|v|
        format!(" ({ship_id}, {project_id}, {}, {})", v.0, v.1)
    ).collect();
    let full_sql = format!(
        "DO $$ BEGIN \
        DELETE FROM parameter_data \
        WHERE ship_id= {ship_id} AND project_id IS NOT DISTINCT FROM {project_id}; \
        INSERT INTO parameter_data \
        (ship_id, project_id, parameter_id, result) \
        VALUES \
        {data_str}; \
        END$$;",
        data_str = data_list.join(", ")
    );
    // println!("{}", &full_sql);
    api_client.fetch(&full_sql).map_err(|err| error.pass(err))?;
    log::info!("send_parameters_data end");
    Ok(())
}
