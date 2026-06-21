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
            entities::Bounds, 
            eval::reports::bonjan_report::{bonjan_report_ctx::BonjanReportCtx, bonjan_report_eval::BonjanReportEval}
        }, 
        kernel::Eval, prelude::{
            Context, 
            ContextRead, 
            InitialCtx
        }, tests::unit::algorithm::reports::bonjan_report_test::MocEval, 
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
        let test_duration = TestDuration::new(dbg, Duration::from_secs(1000));
        test_duration.run().unwrap();
        let test_data = [
            (
                1,
                1.0,
                vec![0.0, 6.5250, 13.0500, 19.5750, 26.1000, 32.6250, 39.1500, 45.6750, 52.2000, 58.7250, 65.2500, 71.7750, 78.3000, 84.8250, 91.3500, 97.875, 104.4000, 110.9250, 117.4500, 123.9750, 130.5000],
                "src\\tests\\unit\\algorithm\\reports\\test_files\\SURF_Sofiya.stl",
                "src\\tests\\unit\\algorithm\\reports\\test_files\\style.css"
            ),
            // (
            //     1,
            //     1.0,
            // vec![
            //     -3.6,
            //     -3.0,
            //     -2.4,
            //     -1.8,
            //     -1.2,
            //     -0.6,
            //     0.0,
            //     0.6,
            //     1.2,
            //     1.8,
            //     2.4,
            //     3.0,
            //     3.6,
            //     4.2,
            //     4.8,
            //     5.4,
            //     6.0,
            //     6.7,
            //     7.4,
            //     8.1,
            //     8.8,
            //     9.5,
            //     10.2,
            //     10.9,
            //     11.6,
            //     12.3,
            //     13.0,
            //     13.7,
            //     14.4,
            //     15.1,
            //     15.8,
            //     16.5,
            //     17.2,
            //     17.9,
            //     18.6,
            //     19.34,
            //     20.08,
            //     20.82,
            //     21.56,
            //     22.3,
            //     23.04,
            //     23.78,
            //     24.52,
            //     25.26,
            //     26.0,
            //     26.74,
            //     27.48,
            //     28.22,
            //     28.96,
            //     29.7,
            //     30.44,
            //     31.18,
            //     31.92,
            //     32.66,
            //     33.4,
            //     34.14,
            //     34.88,
            //     35.62,
            //     36.36,
            //     37.1,
            //     37.84,
            //     38.58,
            //     39.32,
            //     40.06,
            //     40.8,
            //     41.54,
            //     42.28,
            //     43.02,
            //     43.76,
            //     44.5,
            //     45.24,
            //     45.98,
            //     46.72,
            //     47.46,
            //     48.2,
            //     48.94,
            //     49.68,
            //     50.42,
            //     51.16,
            //     51.9,
            //     52.64,
            //     53.38,
            //     54.12,
            //     54.86,
            //     55.6,
            //     56.34,
            //     57.08,
            //     57.82,
            //     58.56,
            //     59.3,
            //     60.04,
            //     60.78,
            //     61.52,
            //     62.26,
            //     63.0,
            //     63.74,
            //     64.48,
            //     65.22,
            //     65.96,
            //     66.7,
            //     67.44,
            //     68.18,
            //     68.92,
            //     69.66,
            //     70.4,
            //     71.14,
            //     71.88,
            //     72.62,
            //     73.36,
            //     74.1,
            //     74.84,
            //     75.58,
            //     76.32,
            //     77.06,
            //     77.8,
            //     78.54,
            //     79.28,
            //     80.02,
            //     80.76,
            //     81.5,
            //     82.24,
            //     82.98,
            //     83.72,
            //     84.46,
            //     85.2,
            //     85.94,
            //     86.68,
            //     87.42,
            //     88.16,
            //     88.9,
            //     89.64,
            //     90.38,
            //     91.12,
            //     91.86,
            //     92.6,
            //     93.34,
            //     94.08,
            //     94.82,
            //     95.56,
            //     96.3,
            //     97.04,
            //     97.78,
            //     98.52,
            //     99.26,
            //     100.0,
            //     100.74,
            //     101.48,
            //     102.22,
            //     102.96,
            //     103.7,
            //     104.44,
            //     105.18,
            //     105.92,
            //     106.66,
            //     107.4,
            //     108.14,
            //     108.88,
            //     109.62,
            //     110.36,
            //     111.1,
            //     111.84,
            //     112.58,
            //     113.32,
            //     114.06,
            //     114.8,
            //     115.54,
            //     116.28,
            //     117.02,
            //     117.76,
            //     118.5,
            //     119.24,
            //     119.98,
            //     120.72,
            //     121.46,
            //     122.2,
            //     122.94,
            //     123.68,
            //     124.42,
            //     125.16,
            //     125.9,
            //     126.5,
            //     127.1,
            //     127.7,
            //     128.3,
            //     128.9,
            //     129.5,
            //     130.1,
            //     130.7,
            //     131.3,
            //     131.9,
            //     132.5,
            //     133.1,
            //     133.7,
            //     134.3,
            //     134.9,
            //     135.5
            // ],
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\SURF_Sofiya.stl",
            //     "src\\tests\\unit\\algorithm\\reports\\test_files\\style.css"
            // )

        ];
        for (step, draught_step, samples, model_ship_path, style_path) in test_data.iter() {
            let initial = InitialCtx::new(
                "0",
                "Unit-test",
                Bounds::from_min_max(0., 100., 20).unwrap(),
            );
            let ctx = MocEval {
                ctx: Context::new(initial),
            };
            match BonjanReportEval::new(
                "dbg", 
                ctx,
                PathBuf::from(&model_ship_path),
                PathBuf::from(style_path),
                *draught_step,
                samples.to_vec(),
            ).eval(()) {
                Ok(ctx) => {
                    let result = ContextRead::<BonjanReportCtx>::read(&ctx).result;
                    std::fs::File::create(format!("src\\tests\\unit\\algorithm\\reports\\output_files\\report_bonjan.html")).unwrap();
                    std::fs::write(format!("src\\tests\\unit\\algorithm\\reports\\output_files\\report_bonjan.html"), result).unwrap();
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