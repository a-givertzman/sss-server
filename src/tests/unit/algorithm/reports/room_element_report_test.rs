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
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{
        DebugSession, 
        LogLevel
    };
    use crate::{
        algorithm::{
            entities::{Bounds, model_cached::load_stl}, 
            eval::room_element_report::{
                room_element_report_ctx::RoomElementReportCtx, 
                room_element_report_eval::RoomElementReportEval
            }
        }, 
        kernel::Eval, prelude::{
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
        let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                5,
                "D:\\work_projects\\sss-server\\src\\tests\\unit\\algorithm\\reports\\test_files\\tanks_1.stl"
            )
        ];
        for (step, id_tank, tank_path) in test_data.iter() {
            let mut initial = InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
            );
            let mut ctx = MocEval {
                ctx: Context::new(initial),
            };
            match load_stl(&PathBuf::from(tank_path)) {
                Ok(tank) => {
                    match RoomElementReportEval::new(
                        "dbg", 
                        ctx,
                        tank
                    ).eval(()) {
                        Ok(ctx) => {
                            let result = ContextRead::<RoomElementReportCtx>::read(&ctx).result;
                            std::fs::File::create("index.html").unwrap();
                            std::fs::write("index.html", result).unwrap();
                            log::debug!("{dbg} | Result html stored into 'index.html'");
                            log::debug!("{dbg} | All done");
                        },
                        Err(err) => panic!("step {} \nerror: {:#?}", step, err),
                    }
                },
                Err(e) => log::error!("Error to load stl file to TriMesh: {:?}", e),
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