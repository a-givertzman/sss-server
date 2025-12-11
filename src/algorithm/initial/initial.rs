use std::sync::Arc;

use super::initial_ctx::InitialCtx;
use crate::algorithm::entities::data::stability::ship_type::ShipType;
use crate::algorithm::entities::data::stability::{BowBoardDataArray, DraftMarkDataArray, IcingArray, LoadLineDataArray, NavigationArea, ScrewDataArray, ship_type};
use crate::algorithm::entities::model_cached::ModelCached;
use crate::algorithm::entities::Bounds;
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::{
    CoefficientKArray, CoefficientKThetaArray, 
    MultiplerSArray, MultiplerX1Array, MultiplerX2Array, 
    loads::*, MetacentricHeightSubdivisionArray,
};
use crate::algorithm::entities::data::{ShipArray, ShipParametersArray, VoyageArray};
use crate::algorithm::entities::ship_model::ship_model::ShipModel;
use crate::kernel::types::RwLock;
use crate::kernel::types::eval_result::EvalResult;
use crate::{
    algorithm::context::{
        context::Context,
        context_access::{ContextReadRef, ContextWrite},
    },
    infrostructure::ApiClient,
    kernel::Eval,
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
pub struct Initial {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    api_client: Arc<ApiClient>,
    ctx: Context,
}
//
impl Initial {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        api_client: Arc<ApiClient>,
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
    fn eval(&self, _: ()) -> EvalResult {     
        let error = Error::new(&self.dbg, "eval");
        let initial_ctx: &InitialCtx = self.ctx.read_ref();
        let mut initial_ctx = initial_ctx.to_owned();
        let ship = ShipArray::parse(
            &self
                .api_client
                .fetch(&format!(
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
                ))
                .map_err(|err| error.pass_with("ship fetch", err))?,
        )
        .map_err(|err| error.pass_with("ship parse", err))?;
        let ship = match ship.data.first() {
            Some(data) => data.to_owned(),
            None => return Err(error.err("Error ship: no first in data")),
        };
        let navigation_area = NavigationArea::from_str(&ship.navigation_area)
            .map_err(|e| error.pass_with("navigation_area", e))?;
        let ship_type =
            ShipType::from_str(&ship.ship_type).map_err(|e| error.pass_with("ship_type", e))?;
        let voyage = VoyageArray::parse(
            &self
                .api_client
                .fetch(&format!(
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
                ))
                .map_err(|err| error.pass_with("voyage fetch", err))?,
        )
        .map_err(|err| error.pass_with("voyage parse", err))?;
        let voyage = match voyage.data.first() {
            Some(data) => data.to_owned(),
            None => return Err(error.err("Error voyage: no first in data")),
        };
        let ship_parameters = ShipParametersArray::parse(&self.api_client.fetch(&format!(
                "SELECT key, value FROM \"ship/ship_general_characteristics\" WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                initial_ctx.ship_id, initial_ctx.project_id
            )).map_err(|err| error.pass_with("ship_parameters fetch", err))?
        ).map_err(|err| error.pass_with("ship_parameters parse", err))?;
        let icing = IcingArray::parse(
            &self
                .api_client
                .fetch(&format!("SELECT key, value FROM icing;"))
                .map_err(|err| error.pass_with("icing fetch", err))?,
        )
        .map_err(|err| error.pass_with("icing parse", err))?;
        let load_constant = LoadConstantArray::parse(
            &self
                .api_client
                .fetch(&format!(
                "SELECT 
                    mass, \
                    bound_x1, \
                    bound_x2
                FROM 
                    \"ship/ship_structures/load_constant\"
                WHERE 
                    ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                    initial_ctx.ship_id, initial_ctx.project_id
                ))
                .map_err(|err| error.pass_with("load_constant fetch", err))?,
        )
        .map_err(|err| error.pass_with("load_constant parse", err))?;
        let bulk = LoadBulkArray::parse(
            &self
                .api_client
                .fetch(&format!(
                "SELECT 
                    space_id, \
                    space_name, \
                    cargo_id, \
                    cargo_name, \
                    assignment_id, \
                    assigment_context as assigment_type, \
                    cargo_type, \
                    stowage_factor, \
                    weight AS mass, \
                    shiftable AS shiftable, \
                    centre_of_compartment_x as mass_shift_x,
                    centre_of_compartment_y as mass_shift_y,
                    centre_of_compartment_z as mass_shift_z
                FROM 
                    bulk_cargo_view
                WHERE 
                    language = 'en' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                    initial_ctx.ship_id, initial_ctx.project_id
                ))
                .map_err(|err| error.pass_with("bulk fetch", err))?,
        )
        .map_err(|err| error.pass_with("bulk parse", err))?;
        let liquid = LoadLiquidArray::parse(
            &self
                .api_client
                .fetch(&format!(
                "SELECT 
                    cargo_id, \
                    cargo_name, \
                    space_id, \
                    space_name, \
                    assignment_id, \
                    assigment_context as assigment_type, \
                    compartment_purpose as compartment_purpose, \
                    cargo_type, \
                    weight AS mass, \
                    density, \
                    volume, \
                    centre_of_compartment_x as mass_shift_x,
                    centre_of_compartment_y as mass_shift_y,
                    centre_of_compartment_z as mass_shift_z,
                    use_moment_of_inertia_max, \
                    long_moment_of_inertia_max, \
                    trans_moment_of_inertia_max
                FROM 
                    liquid_cargo_view
                WHERE 
                    language = 'en' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                    initial_ctx.ship_id, initial_ctx.project_id
                ))
                .map_err(|err| error.pass_with("liquid fetch", err))?,
        )
        .map_err(|err| error.pass_with("liquid parse", err))?;
        let gaseous = LoadGaseousArray::parse(
            &self
                .api_client
                .fetch(&format!(
                "SELECT
                    cargo_id, \
                    cargo_name, \
                    space_id, \
                    space_name, \
                    assignment_id, \
                    assigment_context as assigment_type, \
                    cargo_type, \
                    density, \
                    weight AS mass, \
                    centre_of_compartment_x as mass_shift_x,
                    centre_of_compartment_y as mass_shift_y,
                    centre_of_compartment_z as mass_shift_z
                FROM 
                    gaseous_cargo_view
                WHERE 
                    language = 'en' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                    initial_ctx.ship_id, initial_ctx.project_id
                ))
                .map_err(|err| error.pass_with("gaseous fetch", err))?,
        )
        .map_err(|err| error.pass_with("gaseous parse", err))?;
        let container = LoadContainerArray::parse(
            &self
                .api_client
                .fetch(&format!(
                "SELECT 
                    c.cargo_id AS cargo_id, \
                    c.slot_id AS slot_id, \
                    c.cargo_name AS cargo_name, \
                    c.space_id AS space_id, \
                    c.assignment_id AS assignment_id, \
                    c.assigment_context AS assigment_type, \
                    c.weight AS mass, \
                    c.bound_x1 AS bound_x1, \
                    c.bound_x2 AS bound_x2, \
                    c.bound_y1 AS bound_y1, \
                    c.bound_y2 AS bound_y2, \
                    c.bound_z1 AS bound_z1, \
                    c.bound_z2 AS bound_z2
                FROM 
                    container_cargo_view AS c
                WHERE 
                    language = 'en' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                    initial_ctx.ship_id, initial_ctx.project_id
                ))
                .map_err(|err| error.pass_with("container fetch", err))?,
        )
        .map_err(|err| error.pass_with("container parse", err))?;
        let unit = LoadUnitArray::parse(
            &self
                .api_client
                .fetch(&format!(
                "SELECT 
                    cargo_id, \
                    cargo_name, \
                    space_id, \
                    space_name, \
                    assignment_id, \
                    assigment_context as assigment_type, \
                    cargo_type, \
                    weight AS mass, \
                    centre_of_gravity_x AS mass_shift_x, \
                    centre_of_gravity_y AS mass_shift_y, \
                    centre_of_gravity_z AS mass_shift_z, \
                    stowage_factor, \
                    permeability, \
                    icing_area, \
                    centre_of_icing_area_x, \
                    centre_of_icing_area_y, \
                    centre_of_icing_area_z, \
                    windage_area, \
                    centre_of_windage_area_x, \
                    centre_of_windage_area_y, \
                    centre_of_windage_area_z, \
                    bound_x1, \
                    bound_x2, \
                    bound_y1, \
                    bound_y2, \
                    bound_z1, \
                    bound_z2
                FROM 
                    unit_cargo_view
                WHERE 
                    language = 'en' AND ship_id={} AND project_id IS NOT DISTINCT FROM {};",
                    initial_ctx.ship_id, initial_ctx.project_id
                ))
                .map_err(|err| error.pass_with("unit fetch", err))?,
        )
        .map_err(|err| error.pass_with("unit parse", err))?;
        let mut unit_data = unit.data();
        unit_data.append(&mut container.data());
        let multipler_x1 = MultiplerX1Array::parse(
            &self
                .api_client
                .fetch(&format!("SELECT key, value FROM multipler_x1;"))
                .map_err(|err| error.pass_with("multipler_x1 fetch", err))?,
        )
        .map_err(|err| error.pass_with("multipler_x1 parse", err))?;
        let multipler_x2 = MultiplerX2Array::parse(
            &self
                .api_client
                .fetch(&format!("SELECT key, value FROM multipler_x2;"))
                .map_err(|err| error.pass_with("multipler_x2 fetch", err))?,
        )
        .map_err(|err| error.pass_with("multipler_x2 parse", err))?;
        let multipler_s = MultiplerSArray::parse(
            &self
                .api_client
                .fetch(&format!("SELECT area, t, s FROM multipler_s;"))
                .map_err(|err| error.pass_with("multipler_s fetch", err))?,
        )
        .map_err(|err| error.pass_with("multipler_s parse", err))?;
        let coefficient_k = CoefficientKArray::parse(
            &self
                .api_client
                .fetch(&format!("SELECT key, value FROM coefficient_k;"))
                .map_err(|err| error.pass_with("coefficient_k fetch", err))?,
        )
        .map_err(|err| error.pass_with("coefficient_k parse", err))?;
        let coefficient_k_theta = CoefficientKThetaArray::parse(
            &self
                .api_client
                .fetch(&format!("SELECT key, value FROM coefficient_k_theta;"))
                .map_err(|err| error.pass_with("coefficient_k_theta fetch", err))?,
        )
        .map_err(|err| error.pass_with("coefficient_k_theta parse", err))?;
        let load_line = LoadLineDataArray::parse(&self.api_client.fetch(&format!(
            "SELECT criterion_id, title as name, x, y, z FROM load_line_view WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
            initial_ctx.ship_id, initial_ctx.project_id
        )).map_err(|err| error.pass_with("load_line fetch", err))?
        ).map_err(|err| error.pass_with("load_line parse", err))?;
        let bow_board = BowBoardDataArray::parse(&self.api_client.fetch(&format!(
            "SELECT criterion_id, title as name, x, y, z FROM bow_board_view WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
            initial_ctx.ship_id, initial_ctx.project_id
        )).map_err(|err| error.pass_with("bow_board fetch", err))?
        ).map_err(|err| error.pass_with("bow_board parse", err))?;
        let screw = ScrewDataArray::parse(&self.api_client.fetch(&format!(
            "SELECT criterion_id, x, y, z, d FROM screw_view WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
            initial_ctx.ship_id, initial_ctx.project_id
        )).map_err(|err| error.pass_with("screw fetch", err))?
        ).map_err(|err| error.pass_with("screw parse", err))?;
        let draft_mark = DraftMarkDataArray::parse(&self.api_client.fetch(&format!(
            "SELECT criterion_id, name, x, y, z FROM draft_mark WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
            initial_ctx.ship_id, initial_ctx.project_id
        )).map_err(|err| error.pass_with("draft_mark fetch", err))?
        ).map_err(|err| error.pass_with("draft_mark parse", err))?;
        let h_subdivision = MetacentricHeightSubdivisionArray::parse(&self.api_client.fetch(&format!(
            "SELECT key, value FROM min_metacentric_height_subdivision WHERE ship_id={} AND project_id IS NOT DISTINCT FROM {};",
            initial_ctx.ship_id, initial_ctx.project_id
        )).map_err(|err| error.pass_with("h_subdivision fetch", err))?
        ).map_err(|err| error.pass_with("h_subdivision parse", err))?;
        initial_ctx.ship = Some(ship);
        initial_ctx.ship_type = Some(ship_type);
        initial_ctx.navigation_area = Some(navigation_area);
        initial_ctx.ship_parameters = Some(ship_parameters.data());
        initial_ctx.voyage = Some(voyage);
        initial_ctx.icing = Some(icing);
        initial_ctx.load_constant = Some(load_constant);
        initial_ctx.bulk = Some(bulk.data());
        initial_ctx.liquid = Some(liquid.data());
        initial_ctx.unit = Some(unit_data);
        initial_ctx.gaseous = Some(gaseous.data());
        initial_ctx.multipler_x1 = Some(multipler_x1.data());
        initial_ctx.multipler_x2 = Some(multipler_x2.data());
        initial_ctx.multipler_s = Some(multipler_s);
        initial_ctx.coefficient_k = Some(coefficient_k.data());
        initial_ctx.coefficient_k_theta = Some(coefficient_k_theta);
        initial_ctx.load_line = Some(load_line.load_line_data());
        initial_ctx.bow_board = Some(bow_board.bow_board_data());
        initial_ctx.screw = Some(screw.data());
        initial_ctx.draft_mark = Some(draft_mark.draft_data());
        initial_ctx.h_subdivision = Some(h_subdivision.data());
        self.ctx.clone().write(initial_ctx.to_owned())
    }
}
