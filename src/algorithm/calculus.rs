use std::sync::Arc;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use crate::{algorithm::{entities::ship_model::ship_model::ShipModel, 
    eval::{
        criterion::{
            CriterionDraughtEval, CriterionStabilityEval, acceleration::eval::AccelerationEval, bow_board::eval::BowBoardEval, circulation::eval::CirculationEval, dso_angle_max::eval::DSOAngleMaxEval, dso_area::eval::DSOAreaEval, dso_icing_max::eval::DSOIcingMaxEval, dso_max::eval::DSOMaxEval, dso_timber_max::eval::DSOTimberMaxEval, eval::ResultCriterionEval, grain::eval::GrainEval, load_line::eval::LoadLineEval, metacentric_height_subdivision::eval::MetacentricHeightSubdivisionEval, min_metacentric_height::eval::MinMetacentricHeightEval, reserve_buoyncy::eval::ReserveBuoyncyEval, screw::eval::ScrewEval, static_angle::eval::StaticAngleEval, wheather::eval::WheatherEval
        }, draft_mark::eval::DraftMarkEval, icing_coeff::eval::IcingCoeffEval, icing_timber::eval::IcingTimberEval, icing_timber_bound::eval::IcingTimberBoundEval, stability::{
            balance::eval::StabilityBalanceEval, dynamic_mass::eval::DynamicMassStabEval, icing::eval::IcingStabEval, lever_diagram::eval::LeverDiagramEval, metacentric_height::eval::MetacentricHeightEval, roll_amplitude::eval::RollingAmplitudeEval, roll_period::eval::RollingPeriodEval, static_mass::eval::StaticMassStabEval, wind::eval::WindEval, windage::eval::WindageEval
        }, strength::{
            area::eval::AreaStrEval, balance::eval::StrengthBalanceEval, dynamic_mass::eval::DynamicMassStrEval, icing::eval::IcingStrEval, result::eval::ResultStrEval, static_mass::eval::StaticMassStrEval
        }, unit_area::eval::UnitAreaEval, wetting::eval::WettingEval, zg::eval::ZgEval        
    }}, 
    conf::Conf, 
    infrostructure::ApiClient, 
    kernel::{
        Eval, 
        EvalEx, 
        types::{RwLock, eval_result::EvalResult}
    }, 
    prelude::{Context, Initial, InitialCtx}, 
    server::CalculusQuery
};

///
/// Evaluates entair ship calculations
pub struct Calculus {
    dbg: Dbg,
    conf: Conf,
    api_client: Arc<ApiClient>,
    ship_model: Arc<RwLock<ShipModel>>,
    thread_pool: Arc<ThreadPool>,
}
//
//
impl Calculus {
    ///
    /// Returns [Algorithm] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: Conf,    
        api_client: Arc<ApiClient>,
        ship_model: Arc<RwLock<ShipModel>>,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "Algorithm"),
            conf,
            api_client,
            ship_model,
            thread_pool,
        }
    }
}
//
//
impl EvalEx<CalculusQuery, EvalResult> for Calculus {
    fn eval(&self, query: CalculusQuery) -> EvalResult {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "eval");
        log::debug!("{dbg}.eval | Calculations...");
        let bounds = self.ship_model.read().bounds().map_err(|err| error.pass(err))?;
        let ctx =        
        // criterion   
        CriterionStabilityEval::new(
            &dbg,
            MetacentricHeightSubdivisionEval::new(
                &dbg,
                GrainEval::new(
                    &dbg,
                    CirculationEval::new(
                        &dbg,
                        AccelerationEval::new(
                            &dbg,
                            MinMetacentricHeightEval::new(
                                &dbg,
                                DSOAngleMaxEval::new(
                                    &dbg,
                                    DSOTimberMaxEval::new(
                                        &dbg,
                                        DSOIcingMaxEval::new(
                                            &dbg,
                                            DSOMaxEval::new(
                                                &dbg,
                                                DSOAreaEval::new(
                                                    &dbg,
                                                    StaticAngleEval::new(
                                                        &dbg,
                                                        WheatherEval::new(
                                                            &dbg,
        // stability after ZG
        RollingAmplitudeEval::new(
            &dbg,
            RollingPeriodEval::new(
                &dbg,
                WindEval::new(
                    &dbg,
                    WindageEval::new(
                        &dbg,
                        Arc::clone(&self.ship_model),
                        LeverDiagramEval::new(
                            &dbg,
                            Arc::clone(&self.api_client),
                            Arc::clone(&self.ship_model),
                            MetacentricHeightEval::new(
                                &dbg,
        // strength
    /*    BendingMomentEval::new(
            &dbg,
            ShearForceEval::new(
                &dbg,
                TotalForceEval::new(
                    &dbg,                                                        
     */
            ResultStrEval::new(
                    &dbg,   
                    Arc::clone(&self.api_client),
                    DynamicMassStrEval::new(
                        &dbg,
                        StrengthBalanceEval::new(
                            &dbg,
                            Arc::clone(&self.ship_model),                                
                            StaticMassStrEval::new(
                                &dbg,                                   
                                IcingStrEval::new(
                                    &dbg, 
                                    Arc::clone(&self.ship_model),                                      
                                    AreaStrEval::new(
                                        &dbg,
                                        Arc::clone(&self.ship_model),
        // stability before ZG
        DynamicMassStabEval::new(
            &dbg,
            StabilityBalanceEval::new(
                &dbg,
                Arc::clone(&self.api_client),
                Arc::clone(&self.ship_model),
                StaticMassStabEval::new(
                    &dbg,
                    IcingStabEval::new(
                        &dbg,
                        Arc::clone(&self.ship_model),                
        WettingEval::new(
            &dbg,
            IcingTimberEval::new(
                &dbg,
                IcingTimberBoundEval::new(
                    &dbg,
                    IcingCoeffEval::new(
                        &dbg,
                        UnitAreaEval::new(
                            &dbg,
                            Initial::new(
                                &dbg,
                                Arc::clone(&self.api_client),
                                Context::new(InitialCtx::new(
                                    &query.ship_id.to_string(),
                                    &query.project_id,
                                    bounds,
                                )),
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
    //        ),
    //    ),
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
            ),
        );//.eval(Zg::empty());
        let ctx = 
        ResultCriterionEval::new(
            &dbg,
            Arc::clone(&self.api_client),
            DraftMarkEval::new(
                &dbg,
                CriterionDraughtEval::new(
                    &dbg,
                    ReserveBuoyncyEval::new(
                        &dbg,
                        ScrewEval::new(
                            &dbg,
                            BowBoardEval::new(
                                &dbg,
                                LoadLineEval::new(
                                    &dbg,
                                        ZgEval::new(
                                            Arc::clone(&self.thread_pool),
                                            &dbg,
                                            ctx,
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ).eval(());
        ctx.map_err(|err| error.pass(err))
    }
    //
    //
    fn exit(&self) {
        todo!()
    }
}
//
//
unsafe impl Send for Calculus {}

