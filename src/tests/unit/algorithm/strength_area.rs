#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{algorithm::eval::{StrengthAreaCtx, StrengthAreaEval}, kernel::eval::Eval, prelude::{Context, InitialCtx}, tests::unit::algorithm::fake_initial::FakeInitial};
    use crate::algorithm::context::context_access::ContextRead;

    #[test]
    fn strength_area() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test strength_area";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own("strength_area");
        let ctx = FakeInitial::new(
            dbg.clone(),
            Context::new(
                InitialCtx::new(
                    1,
                    "NULL",
                ),
            )
        );

        let result: StrengthAreaCtx = StrengthAreaEval::new(&dbg, ctx).eval(()).unwrap().read();

        let target =  StrengthAreaCtx {
            mass: 0.,
            mass_shift_x: 0.,
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

     /*  TODO: mass_values assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );*/

        test_duration.exit();
    }
}
