#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{algorithm::{context::context_access::ContextRead, eval::{IcingCtx, IcingEval}}, kernel::eval::Eval, prelude::{Context, InitialCtx}, tests::unit::algorithm::fake_initial::FakeInitial};
    use crate::algorithm::context::context_access::ContextRead;
    
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
            mass: 67.460925,
            mass_shift_x:1.2989097,
            mass_values: Vec::new(),
        };
        assert!(
            result.mass == target.mass,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass,
            target.mass
        );

        assert!(
            result.mass_shift_x == target.mass_shift_x,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_shift_x,
            target.mass_shift_x
        );
        test_duration.exit();
    }
}
