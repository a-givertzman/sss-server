#[cfg(test)]
mod storage {
    use crate::kernel::storage::storage::Storage;
    use api_tools::debug::dbg_id::DbgId;
    use core::f64;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use serde_json::json;
    use std::{collections::HashMap, i64, sync::Once, time::Duration};
    use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
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
    fn init_each() {}
    ///
    /// Testing load() method on simple types
    #[tokio::test(flavor = "multi_thread")]
    async fn load() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("load".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(2));
        test_duration.run().unwrap();
        let path = "./src/tests/unit/kernel/storage/cache/test_1";
        let mut hooks_storage = Storage::new(path);
        let test_data = [
            (01, "test.int.value-1", Value::Int(i64::MIN)),
            (02, "test.int.value-2", Value::Int(-1)),
            (03, "test.int.value-3", Value::Int(0)),
            (04, "test.int.value-4", Value::Int(1)),
            (05, "test.int.value-5", Value::Int(i64::MAX)),
            (06, "test.double.value-1", Value::Double(-f64::MAX)),
            (07, "test.double.value-2", Value::Double(-0.1)),
            (08, "test.double.value-3", Value::Double(0.0)),
            (09, "test.double.value-4", Value::Double(0.2)),
            (10, "test.double.value-5", Value::Double(0.4)),
            (11, "test.double.value-6", Value::Double(1.0)),
            (12, "test.double.value-7", Value::Double(f64::MAX)),
            (13, "test.string.value-1", Value::String("value-1".into())),
            (14, "test.string.value-2", Value::String("value-2".into())),
        ];
        for (step, key, target) in test_data {
            fn load_value(
                dbgid: &DbgId,
                step: usize,
                hooks_storage: &mut Storage,
                key: &str,
            ) -> serde_json::Value {
                match hooks_storage.load(key) {
                    Ok(result) => result,
                    Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
                }
            }
            match &target {
                Value::Bool(target) => {
                    let result = load_value(&dbgid, step, &mut hooks_storage, key);
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::Int(target) => {
                    let result = load_value(&dbgid, step, &mut hooks_storage, key);
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::Real(target) => {
                    let result = load_value(&dbgid, step, &mut hooks_storage, key);
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::Double(target) => {
                    let result = load_value(&dbgid, step, &mut hooks_storage, key);
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::String(target) => {
                    let result = load_value(&dbgid, step, &mut hooks_storage, key);
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing load() method on Map<String, f64>
    #[tokio::test(flavor = "multi_thread")]
    async fn load_map_str_f64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("load_map_str_f64".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/kernel/storage/cache/test_1";
        let mut hooks_storage = Storage::new(path);
        let test_data = [(
            1,
            "test.map.f64",
            HashMap::from([("12.0", 12.0), ("-14.1", -14.1)]),
        )];
        for (step, key, target) in test_data {
            match hooks_storage.load(key) {
                Ok(result) => assert!(
                    result == json!(target),
                    "step {} \nresult: {:?}\ntarget: {:?}",
                    step,
                    result,
                    target
                ),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing load() on Map<String, String>
    #[tokio::test(flavor = "multi_thread")]
    async fn load_map_str_str() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("load_map_str_str".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/kernel/storage/cache/test_1";
        let mut hooks_storage = Storage::new(path);
        let test_data = [(
            1,
            "test.map.str_str",
            HashMap::from([("12.0", "Value 12.0"), ("-14.1", "Value -14.1")]),
        )];
        for (step, key, target) in test_data {
            match hooks_storage.load(key) {
                Ok(result) => assert!(
                    result == json!(target),
                    "step {} \nresult: {:?}\ntarget: {:?}",
                    step,
                    result,
                    target
                ),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing load() on Vec<String>
    #[tokio::test(flavor = "multi_thread")]
    async fn load_vec_str() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("load_vec_str".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/kernel/storage/cache/test_1";
        let mut hooks_storage = Storage::new(path);
        let test_data = [(1, "test.vec.str", vec!["Value 00", "Value 1", "Value 2"])];
        for (step, key, target) in test_data {
            match hooks_storage.load(key) {
                Ok(result) => assert!(
                    result == json!(target),
                    "step {} \nresult: {:?}\ntarget: {:?}",
                    step,
                    result,
                    target
                ),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing load() on Vec<f64>
    #[tokio::test(flavor = "multi_thread")]
    async fn load_vec_f64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("load_vec_f64".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/kernel/storage/cache/test_1";
        let mut hooks_storage = Storage::new(path);
        let test_data = [(1, "test.vec.f64", vec![-0.223, -0.10, 0.0, 0.10, 0.2204])];
        for (step, key, target) in test_data {
            match hooks_storage.load(key) {
                Ok(result) => assert!(
                    result == json!(target),
                    "step {} \nresult: {:?}\ntarget: {:?}",
                    step,
                    result,
                    target
                ),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
        }
        test_duration.exit();
    }
}
