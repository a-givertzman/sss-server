#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use std::{rc::Rc, sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;

    use crate::{algorithm::eval::IcingStabEval, kernel::{dbgid::dbgid::DbgId, eval::Eval}, prelude::{Context, InitialCtx}};

    #[test]
    fn icing_stab() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test icing_stab";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = DbgId("icing_stab_test".into());
        let ctx = Context::new(
            InitialCtx::new(
                2,
                "NULL",
            ),
        );
        let result = IcingStabEval::new(&dbg, ctx.eval()).eval(val).await.unwrap();
        let result = unsafe { ICING.clone().unwrap().mass(&Bound::Full).unwrap() };
        let target =  50.*0.04 + 50.*1.05*0.015;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );

        test_duration.exit();
    }
}
