use std::sync::Arc;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use crate::{algorithm::{entities::{Bounds, ship_model::ship_model::ShipModel}, eval::*}, conf::Conf, infrostructure::ApiClient, kernel::{Eval, EvalEx, types::{RwLock, eval_result::EvalResult}}, prelude::{Context, Initial, InitialCtx}, server::CalculusQuery};

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
        //  let bounds = Bounds::from_array(&physical_frames, model_center_coord.x()).unwrap();
        let bounds = self.ship_model.write().init().unwrap();
        log::debug!("{dbg}.eval | Calculations...");
        let ctx =           
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
                                                            RollingAmplitudeEval::new(
                                                                &dbg,
                                                                RollingPeriodEval::new(
                                                                    &dbg,
                                                                    WindEval::new(
                                                                        &dbg,
                                                                        WindageEval::new(
                                                                            &dbg,
                                                                            self.ship_model.clone(),
                                                                            LeverDiagramEval::new(
                                                                                &dbg,
                                                                                self.ship_model.clone(), 
                                                                                MetacentricHeightEval::new(
                                                                                    &dbg,
                                                                                    // Before ZG
                                                                                    UnitAreaEval::new(
                                                                                        &dbg,
                                                                                        StaticAreaEval::new(
                                                                                            &dbg,
                                                                                            self.ship_model.clone(),                                                                                    
                                                                                            BendingMomentEval::new(
                                                                                                &dbg,
                                                                                                ShearForceEval::new(
                                                                                                    &dbg,
                                                                                                    TotalForceEval::new(
                                                                                                        &dbg,                                                                                
                                                                                                        DynamicMassEval::new(
                                                                                                            &dbg,
                                                                                                            StrengthBalanceEval::new(
                                                                                                                &dbg,
                                                                                                                self.ship_model.clone(),
                                                                                                                StabilityBalanceEval::new(
                                                                                                                    &dbg,
                                                                                                                    self.ship_model.clone(),
                                                                                                                    StaticMassEval::new(
                                                                                                                        &dbg,
                                                                                                                        WettingEval::new(
                                                                                                                            &dbg,
                                                                                                                            IcingEval::new(
                                                                                                                                &dbg,
                                                                                                                                StaticAreaEval::new(
                                                                                                                                    &dbg,
                                                                                                                                    self.ship_model.clone(),
                                                                                                                                    IcingTimberEval::new(
                                                                                                                                        &dbg,
                                                                                                                                        IcingStabEval::new(
                                                                                                                                            &dbg,
                                                                                                                                            Initial::new(
                                                                                                                                                &dbg,
                                                                                                                                                self.api_client.clone(),
                                                                                                                                                Context::new(InitialCtx::new(
                                                                                                                                                    query.ship_id,
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
        let ctx = DraftMarkEval::new(
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
