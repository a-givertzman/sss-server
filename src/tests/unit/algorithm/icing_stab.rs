#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use std::{rc::Rc, sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;

    use crate::{algorithm::{context::context_access::ContextRead, eval::{IcingStabCtx, IcingStabEval}}, kernel::{dbgid::dbgid::DbgId, eval::Eval}, prelude::{Context, InitialCtx}, tests::unit::algorithm::fake_initial::FakeInitial};

    #[test]
    fn icing_stab() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test icing_stab";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = DbgId("icing_stab_test".into());
        let ctx = FakeInitial::new(
            dbg,
            Context::new(
                InitialCtx{
                    ship_id,
                    project_id,
                },
            )
        );

        let result: IcingStabCtx = IcingStabEval::new(&dbg, ctx).eval(()).await.unwrap().read();

        let target =  IcingStabCtx {
            mass_desc_h: todo!(), 
            mass_timber_h: todo!(), 
            mass_v: todo!(), 
            coef_v_area: todo!(), 
            coef_v_ds_area: todo!(), 
            coef_v_moment: todo!(), 
            is_some: todo!() 
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
