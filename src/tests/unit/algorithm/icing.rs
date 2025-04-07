#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use std::{default, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{algorithm::{context::context_access::ContextRead, eval::{IcingCtx, IcingEval}}, kernel::{dbgid::dbgid::DbgId, eval::Eval}, prelude::{Context, InitialCtx}, tests::unit::algorithm::fake_initial::FakeInitial};

    #[test]
    fn icing() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test icing";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = DbgId("icing".into());
        let ctx = FakeInitial::new(
            dbg,
            Context::new(
                InitialCtx{
                    ship_id: "2".to_owned(),
                    project_id: "NULL".to_owned(),                    
                    ..default()
                },
            )
        );

        let result: IcingCtx = IcingEval::new(&dbg, ctx).eval(()).await.unwrap().read();

        let target =  IcingCtx {
            pub mass: f64,
            pub mass_shift_x: f64,
            pub mass_values: Vec<f64>,
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
