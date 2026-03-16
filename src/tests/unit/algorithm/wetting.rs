#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{algorithm::{entities::Position, eval::{WettingCtx, WettingEval}}, kernel::eval::Eval, prelude::{Context, InitialCtx}, tests::unit::algorithm::fake_initial::FakeInitial};
    use crate::algorithm::context::context_access::ContextRead;

    #[test]
    fn wetting() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test wetting";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own("wetting");
        let ctx = FakeInitial::new(
            dbg.clone(),
            Context::new(
                InitialCtx::new(
                    1,
                    "NULL",
                ),
            )
        );

        let result: WettingCtx = WettingEval::new(&dbg, ctx).eval(()).unwrap().read();

        let target =  WettingCtx {
            mass: 0.,
            mass_shift: Position::zero(),
            mass_values: Vec::new(),
        };

        assert!(
            result.mass == target.mass,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass,
            target.mass
        );

        assert!(
            result.mass_shift == target.mass_shift,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_shift,
            target.mass_shift
        );

        test_duration.exit();
    }
}
