#[cfg(test)]

mod tests {
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    use crate::{algorithm::{entities::Position, eval::{LoadsCtx, LoadsEval}}, kernel::eval::Eval, prelude::{Context, InitialCtx}, tests::unit::algorithm::fake_initial::FakeInitial};
    use crate::algorithm::context::context_access::ContextRead;

    #[test]
    fn loads() {
        DebugSession::new().filter(LogLevel::Debug).init();
        let self_id = "test loads";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own("loads");
        let ctx = FakeInitial::new(
            dbg.clone(),
            Context::new(
                InitialCtx::new(
                    1,
                    "NULL",
                ),
            )
        );

        let result: LoadsCtx = LoadsEval::new(&dbg, ctx).eval(()).unwrap().read();

        let target =  LoadsCtx {
            mass_const: 2044.1,
            mass_bulk: 24.,
            mass_liquid: 283.2546,
            mass_unit: 0.,
            mass_gaseous: 3.,
            shift_const: Position::new(1.05, 0., 5.32),
            shift_unit: Position::zero(),
            shift_gaseous: Position::new(43.67, -0.836, 7.88),
            shift_liquid: Position::zero(),
            shift_bulk: Position::zero(),
            bulk: Vec::new(),
            liquid: Vec::new(),
            grain_bulkhead: Vec::new(),
        };

        assert!(
            result.mass_const == target.mass_const,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_const,
            target.mass_const
        );

        assert!(
            result.mass_bulk == target.mass_bulk,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_bulk,
            target.mass_bulk
        );

        assert!(
            result.mass_liquid == target.mass_liquid,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_liquid,
            target.mass_liquid
        );

        assert!(
            result.mass_unit == target.mass_unit,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_unit,
            target.mass_unit
        );

        assert!(
            result.mass_gaseous == target.mass_gaseous,
            "\nresult: {:?}\ntarget: {:?}",
            result.mass_gaseous,
            target.mass_gaseous
        );

        assert!(
            result.shift_const == target.shift_const,
            "\nresult: {:?}\ntarget: {:?}",
            result.shift_const,
            target.shift_const
        );

        assert!(
            result.shift_unit == target.shift_unit,
            "\nresult: {:?}\ntarget: {:?}",
            result.shift_unit,
            target.shift_unit
        );

        assert!(
            result.shift_gaseous == target.shift_gaseous,
            "\nresult: {:?}\ntarget: {:?}",
            result.shift_gaseous,
            target.shift_gaseous
        );
        
        test_duration.exit();
    }
}
