use crate::{
    kernel::{
        Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::Context
};
#[cfg(test)]
mod tests_eval {
    use std::{
        path::PathBuf, sync::Once, time::Duration
    };
    use parking_lot::RwLock;
    use sal_core::dbg::Dbg;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{
        DebugSession, 
        LogLevel
    };
    use crate::{
        algorithm::{
            entities::{Bounds, model_cached::{DisplacementShape, Shape}}, 
            eval::room_element_report::{
                room_element_report_ctx::RoomElementReportCtx, 
                room_element_report_eval::RoomElementReportEval
            }
        }, 
        kernel::{Eval, types::Arc}, prelude::{
            Context, 
            ContextRead, 
            InitialCtx
        }, 
        tests::unit::algorithm::reports::room_element_report_test::MocEval
    };
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing 'eval'
    #[test]
    fn eval() {
        DebugSession::new().filter(LogLevel::Info).init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = "eval";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1000));
        test_duration.run().unwrap();
        let test_data = [
            // (
            //     1,
            //     1,
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\tanks_1.stl",
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\style.css"
            // ),
            // (
            //     2,
            //     2,
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\502.stl",
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\style.css"
            // ),
            // (
            //     3,
            //     2,
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\221_P.stl",
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\style.css"
            // ),
            (
                3,
                2,
                vec![0.0, 0.0, 0.0, 2.0, 2.0, 2.0, 5.0, 5.0, 5.0],
                vec![0.0, 5.0, 10.0, 0.0, 5.0, 10.0, 0.0, 5.0, 10.0],
                "src\\tests\\unit\\algorithm\\reports\\test_files\\305_P.stl",
                "src\\tests\\unit\\algorithm\\reports\\test_files\\style.css",
            ),
        ];
        for (step, id_tank, heel, trim, tank_path, style_path) in test_data.iter() {
            let initial = InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
            );
            let ctx = MocEval {
                ctx: Context::new(initial),
            };
            let mut displacement_shape = DisplacementShape::new_uninit(
                &Dbg::new("Test", "RoomElementReportEval"), 
                tank_path.into(), 
                None, 
                1.0,
            );
            let _ = displacement_shape.init();
            match RoomElementReportEval::new(
                "dbg", 
                ctx,
                PathBuf::from(style_path),
                Arc::new(RwLock::new(displacement_shape)),
                heel.to_vec(),
                trim.to_vec(),
            ).eval(()) {
                Ok(ctx) => {
                    let result = ContextRead::<RoomElementReportCtx>::read(&ctx).result;
                    std::fs::File::create(format!("src\\tests\\unit\\algorithm\\reports\\output_files\\report.html")).unwrap();
                    std::fs::write(format!("src\\tests\\unit\\algorithm\\reports\\output_files\\report.html"), result).unwrap();
                    log::debug!("{dbg} | Result html stored into 'index.html'");
                    log::debug!("{dbg} | All done");
                },
                Err(err) => panic!("step {} \nerror: {:#?}", step, err),
            }
        }
        test_duration.exit();
    }
}
///
///
#[derive(Debug, Clone)]
struct MocEval {
    pub ctx: Context,
}
//
//
impl Eval<(), EvalResult> for MocEval {
    fn eval(&self, _: ()) -> EvalResult {
        Result::Ok(self.ctx.clone())
    }
}