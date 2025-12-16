use std::sync::Arc;
use sal_core::{dbg::Dbg, error::Error};
use crate::{algorithm::{entities::{Bounds, ship_model::ship_model::ShipModel}, eval::*}, conf::Conf, infrostructure::ApiClient, kernel::{Eval, EvalEx, types::{RwLock, eval_result::EvalResult}}, prelude::{Context, Initial, InitialCtx}, server::CalculusQuery};

///
/// Evaluates entair ship calculations
pub struct Calculus {
    dbg: Dbg,
    conf: Conf,
    api_client: Arc<ApiClient>,
    ship_model: Arc<RwLock<ShipModel>>,
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
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "Algorithm"),
            conf,
            api_client,
            ship_model,
        }
    }
    pub const PHYSICAL_FRAMES: [f64; 196] = [
        -3.6, -3.0, -2.4, -1.8, -1.2, -0.6, 0.0, 0.6, 1.2, 1.8, 2.4, 3.0, 3.6, 4.2, 4.8, 5.4, 6.0,
        6.7, 7.4, 8.1, 8.8, 9.5, 10.2, 10.9, 11.6, 12.3, 13.0, 13.7, 14.4, 15.1, 15.8, 16.5, 17.2,
        17.9, 18.6, 19.34, 20.08, 20.82, 21.56, 22.3, 23.04, 23.78, 24.52, 25.26, 26.0, 26.74,
        27.48, 28.22, 28.96, 29.7, 30.44, 31.18, 31.92, 32.66, 33.4, 34.14, 34.88, 35.62, 36.36,
        37.1, 37.84, 38.58, 39.32, 40.06, 40.80, 41.54, 42.28, 43.02, 43.76, 44.5, 45.24, 45.98,
        46.72, 47.46, 48.2, 48.94, 49.68, 50.42, 51.16, 51.9, 52.64, 53.38, 54.12, 54.86, 55.6,
        56.34, 57.08, 57.82, 58.56, 59.30, 60.04, 60.78, 61.52, 62.26, 63.0, 63.74, 64.48, 65.22,
        65.96, 66.7, 67.44, 68.18, 68.92, 69.66, 70.4, 71.14, 71.88, 72.62, 73.36, 74.1, 74.84,
        75.58, 76.32, 77.06, 77.8, 78.54, 79.28, 80.02, 80.76, 81.5, 82.24, 82.98, 83.72, 84.46,
        85.2, 85.94, 86.68, 87.42, 88.16, 88.9, 89.64, 90.38, 91.12, 91.86, 92.6, 93.34, 94.08,
        94.82, 95.56, 96.3, 97.04, 97.78, 98.52, 99.26, 100.0, 100.74, 101.48, 102.22, 102.96,
        103.7, 104.44, 105.18, 105.92, 106.66, 107.4, 108.14, 108.88, 109.62, 110.36, 111.1,
        111.84, 112.58, 113.32, 114.06, 114.8, 115.54, 116.28, 117.02, 117.76, 118.5, 119.24,
        119.98, 120.72, 121.46, 122.2, 122.94, 123.68, 124.42, 125.16, 125.9, 126.5, 127.1, 127.7,
        128.3, 128.9, 129.5, 130.1, 130.7, 131.3, 131.9, 132.5, 133.1, 133.7, 134.3, 134.9, 135.5,
    ];
}
//
//
impl EvalEx<CalculusQuery, EvalResult> for Calculus {
    fn eval(&self, query: CalculusQuery) -> EvalResult {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "eval");
        //  let bounds = Bounds::from_array(&physical_frames, model_center_coord.x()).unwrap();
        let bounds = Bounds::from_array(&Self::PHYSICAL_FRAMES, 0.).unwrap();
        log::debug!("{dbg}.eval | Calculations...");
        let ctx = 
        /*  
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
                                                                            LeverDiagramEval::new(
                                                                                &dbg,
                                                                                //   link,
                                                                                MetacentricHeightEval::new(
                                                                                    &dbg,
                                                                                    // Before ZG
                                                                                    StabilityAreaEval::new(
                                                                                        &dbg,
                                                                                        ship_model.clone(),
                                                                                        */

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
                                                                                                                        self.ship_model.clone(),
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
                                                                    ).eval(());
        /*                                                                             ),
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
        )
        */

        /*    
        let _result = DraftMarkEval::new(
            &tmp_dbg,
            CriterionDraughtEval::new(
                &tmp_dbg,
                ReserveBuoyncyEval::new(
                    &tmp_dbg,
                    ScrewEval::new(
                        &tmp_dbg,
                        BowBoardEval::new(
                            &tmp_dbg,
                            LoadLineEval::new(
                                &tmp_dbg,
                                ZgEval::new(
                                        thread_pool.scheduler(),
                                        &tmp_dbg,
                                //      &self.ship_model,
                                        ctx,
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
        .eval(());*/

        // let initial: &InitialCtx = ctx.as_ref();
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
