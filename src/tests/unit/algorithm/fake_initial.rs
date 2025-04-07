
use crate::algorithm::entities::data::loads::*;
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::Bounds;
use crate::prelude::InitialCtx;
use crate::ship_model::model_link::ModelLink;
use crate::{
    algorithm::context::{
            context::Context,
            context_access::{ContextReadRef, ContextWrite},
            ctx_result::CtxResult,
        },
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
};
use sal_sync::services::entity::error::str_err::StrErr;

use super::data::*;


///
/// Заглушка для тестирования, имитирует ввод данных. Содержит все данные
/// для расчетов.
#[derive(Debug)]
pub struct FakeInitial {
    dbg: DbgId,
    ctx: Context,
}
//
//
impl FakeInitial {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: Context,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "FakeInitial");
        Self {
            dbg,
            ctx,
        }
    }
    //
    //
}
//
impl Eval<(), EvalResult> for FakeInitial {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            let initial_ctx: &InitialCtx = self.ctx.read_ref();
            let mut initial_ctx = initial_ctx.to_owned();
            // Расчет баланса в модели
            let bounds = Bounds::from_min_max(-3.6, 135.5, 200).unwrap();
            let ship = ship();
            let ship_parameters = ship_parameters();
            let voyage = voyage();
            let icing = icing();
            let load_constant = load_constant::load_constant();
    
            let data = self.api_client.fetch(&format!(
                "SELECT 
                    space_id, \
                    space_name, \
                    cargo_id, \
                    cargo_name, \
                    assigned_id, \
                    assigment_context as assigment_type, \
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
                    return CtxResult::Err(StrErr(format!("{}.eval | Error bulk: {err}", self.dbg)))
                }
            };
            let data = self.api_client.fetch(&format!(
                "SELECT 
                    space_id, \
                    space_name, \
                    cargo_id, \
                    cargo_name, \
                    assigned_id, \
                    assigment_context as assigment_type, \
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
                    assigment_context as assigment_type, \
                    cargo_type, \
                    density, \
                    weight AS mass, \
                    centre_of_compartment as mass_shift
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
            let data = self.api_client.fetch(&format!(
                "SELECT 
                    space_id, \
                    space_name, \
                    cargo_id, \
                    cargo_name, \
                    assigned_id, \
                    assigment_context as assigment_type, \
                    cargo_type, \
                    density, \
                    weight AS mass, \
                    centre_of_gravity AS mass_shift, \
                    permeability, \
                    stowage_factor, \
                    icing_area, \
                    centre_of_icing_area, \
                    windage_area, \
                    centre_of_windage_area, \
                    bound_x1, \
                    bound_x2, \
                    bound_y1, \
                    bound_y2, \
                    bound_z1, \
                    bound_z2
                FROM 
                    unit_cargo_view
                WHERE 
                    language = 'eng' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
            ));
            let unit = match data {
                Ok(data) => match LoadUnitArray::parse(&data) {
                    Ok(data) => data,
                    Err(err) => {
                        return CtxResult::Err(StrErr(format!(
                            "{}.eval | Error unit: {err}",
                            self.dbg
                        )))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(StrErr(format!("{}.eval | Error unit: {err}", self.dbg)))
                }
            };
            initial_ctx.bounds = Some(bounds);
            initial_ctx.ship = Some(ship);
            initial_ctx.ship_parameters = Some(ship_parameters.data());
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
