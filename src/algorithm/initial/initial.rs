use std::collections::HashMap;

use super::initial_ctx::InitialCtx;
use crate::algorithm::entities::data::loads::*;
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
                "SELECT index, start_x, end_x FROM computed_frame_space WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
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
                    name, \
                    ship_type, \
                    navigation_area, \
                    p_v, \
                    m, \
                    freeboard_type      
                FROM             
                    ship_view     
                WHERE  
                    id = {};",
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
                    density, \
                    operational_speed, \
                    wetting_timber, \
                    icing_type::TEXT, \
                    icing_timber_type::TEXT
                FROM             
                    voyage_view            
                WHERE  
                    ship_id = {} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
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
                "SELECT key, value FROM \"ship/ship_general_characteristics\" WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.ship_id
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
            let data = self.api_client.fetch(&format!(
                "SELECT 
                    mass, \
                    bound_x1, \
                    bound_x2
                FROM 
                    \"ship/ship_structures/load_constant\"
                WHERE 
                    ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
            ));
            let load_constant = match data {
                Ok(data) => match LoadConstantArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error load_constant: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error load_constant: {err}",
                        self.dbg
                    )))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT 
                    space_id, \
                    space_name, \
                    cargo_id, \
                    cargo_name, \
                    assigned_id, \
                    assigment_type, \
                    cargo_type, \
                    stowage_factor, \
                    weight AS mass
                FROM 
                    bulk_cargo_view
                WHERE 
                    language = 'eng' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
            ));
            let bulk = match data {
                Ok(data) => match LoadBulkArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error bulk: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error bulk: {err}",
                        self.dbg
                    )))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT 
                    space_id, \
                    space_name, \
                    cargo_id, \
                    cargo_name, \
                    assigned_id, \
                    assigment_type, \
                    cargo_type, \
                    density, \
                    weight AS mass
                FROM 
                    liquid_cargo_view
                WHERE 
                    language = 'eng' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
            ));
            let liquid = match data {
                Ok(data) => match LoadLiquidArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error liquid: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error liquid: {err}",
                        self.dbg
                    )))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT 
                    space_id, \
                    space_name, \
                    cargo_id, \
                    cargo_name, \
                    assigned_id, \
                    assigment_type, \
                    cargo_type, \
                    density, \
                    weight AS mass
                FROM 
                    gaseous_cargo_view
                WHERE 
                    language = 'eng' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
            ));
            let gaseous = match data {
                Ok(data) => match LoadGaseousArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error gaseous: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!(
                        "{}.eval | Error gaseous: {err}",
                        self.dbg
                    )))
                }
            };
            // TODO
            let unit = DataArray::<LoadUnitData>{ data: Vec::new(), error: HashMap::new()};
            initial_ctx.bounds = Some(bounds.data());
            initial_ctx.ship = Some(ship);
            initial_ctx.ship_parameters = Some(ship_parameters);
            initial_ctx.voyage = Some(voyage);
            initial_ctx.icing = Some(icing);
            initial_ctx.load_constant = Some(load_constant);
            initial_ctx.bulk = Some(bulk);
            initial_ctx.liquid = Some(liquid);
            initial_ctx.unit = Some(unit);
            initial_ctx.gaseous = Some(gaseous);
            self.ctx.clone().write(initial_ctx.to_owned())
        })
    }
}
