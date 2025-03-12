use super::initial_ctx::InitialCtx;
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::{IcingArray, ShipArray, ShipParametersArray, VoyageArray};
use crate::{
    algorithm::{
        context::{
            context::Context,
            context_access::{ContextReadRef, ContextWrite},
            ctx_result::CtxResult,
        },
        entities::data::{ComputedFrameData, ComputedFrameDataArray, DataArray},
    },
    infrostructure::api::client::api_client::ApiClient,
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
};
use sal_sync::services::entity::error::str_err::StrErr;

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug)]
pub struct Initial {
    dbg: DbgId,
    api_client: ApiClient,
    ctx: Context,
}
//
//
impl Initial {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(parent: impl Into<String>, api_client: ApiClient, ctx: Context) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "Initial");
        Self {
            dbg,
            api_client,
            ctx,
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for Initial {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            let initial_ctx: &InitialCtx = self.ctx.read_ref();
            let mut initial_ctx = initial_ctx.to_owned();
            /*
                        let bounds = self.api_client.fetch(&format!(
                            "SELECT index, start_x, end_x FROM computed_frame_space WHERE ship_id={};",
                            initial_ctx.ship_id
                        ));
                        match bounds {
                            Ok(bounds) => match ComputedFrameDataArray::parse(&bounds) {
                                Ok(bounds) => {
                                    let bounds: DataArray<ComputedFrameData> = bounds;
                                    initial_ctx.bounds = Some(bounds.data());
                                    self.ctx.clone().write(initial_ctx.to_owned())
                                }
                                Err(err) => CtxResult::Err(StrErr(format!("{}.eval | Error bounds: {err}", self.dbg))),
                            },
                            Err(err) => CtxResult::Err(StrErr(format!("{}.eval | Error bounds: {err}", self.dbg))),
                        }
            */
            let data = self.api_client.fetch(&format!(
                "SELECT index, start_x, end_x FROM computed_frame_space WHERE ship_id={};",
                initial_ctx.ship_id
            ));
            let bounds = match data {
                Ok(data) => match ComputedFrameDataArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error bounds: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error bounds: {err}",
                        self.dbg
                    )))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT             
                    s.name AS name, \
                    tr.title_eng AS ship_type, \
                    n.area::TEXT AS navigation_area, \
                    n.p_v AS p_v, \
                    n.m AS m, \
                    s.freeboard_type AS freeboard_type      
                FROM             
                   ship AS s        
                JOIN             
                    ship_type AS t ON s.ship_type_id = t.id        
                JOIN             
                    ship_type_rmrs AS tr ON t.type_rmrs = tr.id         
                JOIN             
                    navigation_area AS n ON s.navigation_area_id = n.id        
                WHERE  
                    s.id={};",
                initial_ctx.ship_id
            ));
            let ship = match data {
                Ok(data) => match ShipArray::parse(&data) {
                    Ok(data) => match data.data.first() {
                        Some(data) => data.to_owned(),
                        None => {
                            return CtxResult::Err(StrErr(format!("{}.eval | Error ship: no data", self.dbg)))
                        }
                    },
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error ship: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!("{}.eval | Error ship: {err}", self.dbg)))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT             
                    v.density AS density, \
                    v.operational_speed AS operational_speed, \
                    v.wetting_timber AS wetting_timber, \
                    i.icing_type::TEXT AS icing_type, \
                    it.icing_type::TEXT AS icing_timber_type
                FROM             
                    voyage AS v           
                JOIN             
                    ship_icing AS i ON v.icing_type_id = i.id        
                JOIN             
                    ship_icing_timber AS it ON v.icing_timber_type_id = it.id            
                WHERE  
                    s.id={};",
                initial_ctx.ship_id
            ));
            let voyage = match data {
                Ok(data) => match VoyageArray::parse(&data) {
                    Ok(data) => match data.data.first() {
                        Some(data) => data.to_owned(),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Error voyage: no data",
                                self.dbg
                            )))
                        }
                    },
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error voyage: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error voyage: {err}",
                        self.dbg
                    )))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT key, value FROM ship_parameters WHERE ship_id={};",
                initial_ctx.ship_id
            ));
            let ship_parameters = match data {
                Ok(data) => match ShipParametersArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error ship_parameters: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error ship_parameters: {err}",
                        self.dbg
                    )))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT key, value FROM icing;",
            ));
            let icing = match data {
                Ok(data) => match IcingArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error icing: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error icing: {err}",
                        self.dbg
                    )))
                }
            };
            initial_ctx.bounds = Some(bounds.data());
            initial_ctx.ship = Some(ship);
            initial_ctx.ship_parameters = Some(ship_parameters);
            initial_ctx.voyage = Some(voyage);
            initial_ctx.icing = Some(icing);
            self.ctx.clone().write(initial_ctx.to_owned())
        })
    }
}
