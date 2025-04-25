use super::initial_ctx::InitialCtx;
use crate::algorithm::entities::data::{loads::*, CoefficientKArray, CoefficientKThetaArray, MultiplerSArray, MultiplerX1Array, MultiplerX2Array};
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::{IcingArray, ShipArray, ShipParametersArray, VoyageArray};
use crate::algorithm::entities::Bounds;
use crate::kernel::sync::Link;
use crate::ship_model::query;
use crate::{
    algorithm::context::{
            context::Context,
            context_access::{ContextReadRef, ContextWrite},
            ctx_result::CtxResult,
        },
    infrostructure::api::client::api_client::ApiClient,
    kernel::{eval::Eval, types::eval_result::EvalResult},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug)]
pub struct Initial {
    dbg: Dbg,
    model: Link,
    api_client: ApiClient,
    ctx: Context,
}
//
impl Initial {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        model: Link, 
        api_client: ApiClient,
        ctx: Context,
    ) -> Self {
        let dbg = Dbg::new(parent, "Initial");
        Self {
            dbg,
            model,
            api_client,
            ctx,
        }
    }
}
//
impl Eval<(), EvalResult> for Initial {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        let initial_ctx: &InitialCtx = self.ctx.read_ref();
        let mut initial_ctx = initial_ctx.to_owned();
        // Расчет баланса в модели
        let bounds: Bounds = match self.model.call(query::Query::Bounds) {
            Ok(bounds) => bounds,
            Err(err) => return CtxResult::Err(error.pass_with("model.bounds error", err)),
        };
        
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
                            Err(err) => CtxResult::Err(error.pass_with("Error bounds", err)),
                        },
                        Err(err) => CtxResult::Err(error.pass_with("Error bounds", err)),
                    }
        */
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
                        return CtxResult::Err(error.err("Error ship: no data"))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error ship", err))
                }
            }
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error ship", err))
            }
        };
        let data = self.api_client.fetch(&format!(
            "SELECT             
                density, \
                operational_speed, \
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
                        return CtxResult::Err(error.err("Error voyage: no data"))
                    }
                },
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error voyage", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error voyage", err))
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
                    return CtxResult::Err(error.pass_with("Error ship_parameters", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error ship_parameters", err))
            }
        };
        let data = self
            .api_client
            .fetch(&format!("SELECT key, value FROM icing;",));
        let icing = match data {
            Ok(data) => match IcingArray::parse(&data) {
                Ok(data) => data,
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error icing", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error icing", err))
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
                    return CtxResult::Err(error.pass_with("Error load_constant", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error load_constant", err))
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
                    return CtxResult::Err(error.pass_with("Error bulk", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error bulk", err))
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
                    return CtxResult::Err(error.pass_with("Error liquid", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error liquid", err))
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
                    return CtxResult::Err(error.pass_with("Error gaseous", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error gaseous", err))
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
                    return CtxResult::Err(error.pass_with("Error unit", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error unit", err))
            }
        };
        let data = self.api_client.fetch(&format!("SELECT key, value FROM multipler_x1;"));
        let multipler_x1 = match data {
            Ok(data) => match MultiplerX1Array::parse(&data) {
                Ok(data) => data,
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error multipler_x1", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error multipler_x1", err))
            }
        };
        let data = self.api_client.fetch(&format!("SELECT key, value FROM multipler_x2;"));
        let multipler_x2 = match data {
            Ok(data) => match MultiplerX2Array::parse(&data) {
                Ok(data) => data,
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error multipler_x2", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error multipler_x2", err))
            }
        };
        let data = self.api_client.fetch(&format!("SELECT area, t, s FROM multipler_s;"));
        let multipler_s = match data {
            Ok(data) => match MultiplerSArray::parse(&data) {
                Ok(data) => data,
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error multipler_s", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error multipler_s", err))
            }
        };
        let data = self.api_client.fetch(&format!("SELECT key, value FROM coefficient_k;"));
        let coefficient_k = match data {
            Ok(data) => match CoefficientKArray::parse(&data) {
                Ok(data) => data,
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error coefficient_k", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error coefficient_k", err))
            }
        };
        let data = self.api_client.fetch(&format!("SELECT key, value FROM coefficient_k_theta;"));
        let coefficient_k_theta = match data {
            Ok(data) => match CoefficientKThetaArray::parse(&data) {
                Ok(data) => data,
                Err(err) => {
                    return CtxResult::Err(error.pass_with("Error coefficient_k_theta", err))
                }
            },
            Err(err) => {
                return CtxResult::Err(error.pass_with("Error coefficient_k_theta", err))
            }
        };
        initial_ctx.bounds = Some(bounds);
        initial_ctx.ship = Some(ship);
        initial_ctx.ship_parameters = Some(ship_parameters.data());
        initial_ctx.voyage = Some(voyage);
        initial_ctx.icing = Some(icing);
        initial_ctx.load_constant = Some(load_constant);
        initial_ctx.bulk = Some(bulk.data());
        initial_ctx.liquid = Some(liquid.data());
        initial_ctx.unit = Some(unit.data());
        initial_ctx.gaseous = Some(gaseous.data());
        initial_ctx.multipler_x1 = Some(multipler_x1.data());
        initial_ctx.multipler_x2 = Some(multipler_x2.data());
        initial_ctx.multipler_s = Some(multipler_s);
        initial_ctx.coefficient_k = Some(coefficient_k.data());
        initial_ctx.coefficient_k_theta = Some(coefficient_k_theta);
        self.ctx.clone().write(initial_ctx.to_owned())
    }
}
