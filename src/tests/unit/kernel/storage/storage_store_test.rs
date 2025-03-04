#[cfg(test)]
mod storage {
    use api_tools::debug::dbg_id::DbgId;
    use core::f64;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use serde::Serialize;
    use serde_json::json;
    use std::{
        collections::HashMap, fs::OpenOptions, i64, io::BufReader, path::PathBuf, sync::Once,
        time::Duration,
    };
    use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};

    use crate::kernel::{storage::storage::Storage, str_err::str_err::StrErr};
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
    /// Returns content of JSON file
    fn read(dbgid: &DbgId, path: PathBuf) -> Result<serde_json::Value, StrErr> {
        let file = OpenOptions::new().read(true).open(&path).map_err(|err| {
            StrErr(format!(
                "{}.read | Failed to open file: {:?}, error: {}",
                dbgid, path, err
            ))
        })?;
        match serde_json::from_reader::<_, serde_json::Value>(BufReader::new(file)) {
            Ok(json_value) => Ok(json_value),
            Err(err) => Err(StrErr(format!(
                "{}.read | Parse error: {} in the file: {:?}",
                dbgid, err, path
            ))),
        }
    }
    ///
    /// Testing `store()` method on simple types
    #[tokio::test(flavor = "multi_thread")]
    async fn store() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("store".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/kernel/storage/cache/test_1";
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
            fn store_value(
                dbgid: &DbgId,
                step: usize,
                hooks_storage: &mut Storage,
                key: &str,
                value: impl Serialize,
            ) {
                match hooks_storage.store(key, value) {
                    Ok(_) => log::debug!("{} | step {}: Value succesfully stored", dbgid, step),
                    Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
                }
            }
            match &target {
                Value::Bool(target) => {
                    store_value(&dbgid, step, &mut hooks_storage, key, target);
                    let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::Int(target) => {
                    store_value(&dbgid, step, &mut hooks_storage, key, target);
                    let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::Real(target) => {
                    store_value(&dbgid, step, &mut hooks_storage, key, target);
                    let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::Double(target) => {
                    store_value(&dbgid, step, &mut hooks_storage, key, target);
                    let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
                    assert!(
                        result == json!(target),
                        "step {} \nresult: {:?}\ntarget: {:?}",
                        step,
                        result,
                        target
                    );
                }
                Value::String(target) => {
                    store_value(&dbgid, step, &mut hooks_storage, key, target);
                    let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
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
    /// Testing `store()` method on Map<String, f64>
    #[tokio::test(flavor = "multi_thread")]
    async fn store_map_str_f64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("store_map_str_f64".into());
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
            match hooks_storage.store(key, target.clone()) {
                Ok(_) => log::debug!("{} | step {}: Value succesfully stored", dbgid, step),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
            let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
            assert!(
                result == json!(target),
                "step {} \nresult: {:?}\ntarget: {:?}",
                step,
                result,
                target
            );
        }
        test_duration.exit();
    }
    ///
    /// Testing storing method on Map<String, String>
    #[tokio::test(flavor = "multi_thread")]
    async fn store_map_str_str() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("store_map_str_str".into());
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
            match hooks_storage.store(key, target.clone()) {
                Ok(_) => log::debug!("{} | step {}: Value succesfully stored", dbgid, step),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
            let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
            assert!(
                result == json!(target),
                "step {} \nresult: {:?}\ntarget: {:?}",
                step,
                result,
                target
            );
        }
        test_duration.exit();
    }
    ///
    /// Testing store() on Vec<String>
    #[tokio::test(flavor = "multi_thread")]
    async fn store_vec_str() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("store_vec_str".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/kernel/storage/cache/test_1";
        let mut hooks_storage = Storage::new(path);
        let test_data = [(1, "test.vec.str", vec!["Value 00", "Value 1", "Value 2"])];
        for (step, key, target) in test_data {
            match hooks_storage.store(key, target.clone()) {
                Ok(_) => log::debug!("{} | step {}: Value succesfully stored", dbgid, step),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
            let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
            assert!(
                result == json!(target),
                "step {} \nresult: {:?}\ntarget: {:?}",
                step,
                result,
                target
            );
        }
        test_duration.exit();
    }
    ///
    /// Testing store() on Vec<f64>
    #[tokio::test(flavor = "multi_thread")]
    async fn store_vec_f64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbgid = DbgId("store_vec_f64".into());
        log::debug!("\n{}", dbgid);
        let test_duration = TestDuration::new(&dbgid, Duration::from_secs(1));
        test_duration.run().unwrap();
        let path = "src/tests/unit/kernel/storage/cache/test_1";
        let mut hooks_storage = Storage::new(path);
        let test_data = [(1, "test.vec.f64", vec![-0.223, -0.10, 0.0, 0.10, 0.2204])];
        for (step, key, target) in test_data {
            match hooks_storage.store(key, target.clone()) {
                Ok(_) => log::debug!("{} | step {}: Value succesfully stored", dbgid, step),
                Err(err) => panic!("{} | step {},  Error: {:#?}", dbgid, step, err),
            }
            let result = read(&dbgid, PathBuf::from(path).join(key)).unwrap();
            assert!(
                result == json!(target),
                "step {} \nresult: {:?}\ntarget: {:?}",
                step,
                result,
                target
            );
        }
        test_duration.exit();
    }
}
