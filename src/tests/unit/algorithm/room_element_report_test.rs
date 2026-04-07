use crate::{kernel::{Eval, types::eval_result::EvalResult}, prelude::Context};
#[cfg(test)]
mod tests_eval {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use crate::{algorithm::{entities::Bounds, eval::room_element_report::{room_element_report_ctx::RoomElementReportCtx, room_element_report_eval::RoomElementReportEval}}, kernel::Eval, prelude::{Context, ContextRead, InitialCtx}, tests::unit::algorithm::room_element_report_test::MocEval};
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
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                5
            )
        ];
        for (step, id_tank) in test_data.iter() {
            let mut initial = InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
            );
            let mut ctx = MocEval {
                ctx: Context::new(initial),
            };
            match RoomElementReportEval::new("dbg", ctx).eval(()) {
                Ok(ctx) => {
                    let result = ContextRead::<RoomElementReportCtx>::read(&ctx).result;
                    std::fs::File::create("index.html").unwrap();
                    std::fs::write("index.html", result).unwrap();
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