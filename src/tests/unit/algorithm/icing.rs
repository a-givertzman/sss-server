#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{algorithm::{context::context_access::ContextRead, eval::{IcingCtx, IcingEval}}, kernel::eval::Eval, prelude::{Context, InitialCtx}, tests::unit::algorithm::fake_initial::FakeInitial};

    #[test]
    fn icing() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let dbg = Dbg::own("test icing");
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(10));
        test_duration.run().unwrap();
        let ctx = FakeInitial::new(
            &dbg,
            Context::new(
                InitialCtx::new(
                    1,
                    "NULL",
                ),
            )
        );
        let result: IcingCtx = IcingEval::new(&dbg, ctx).eval(()).unwrap().read();
        let target =  IcingCtx {
            mass: 0.,
            mass_shift_x: 0.,
            mass_values: Vec::new(),
        };
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        test_duration.exit();
    }
}
