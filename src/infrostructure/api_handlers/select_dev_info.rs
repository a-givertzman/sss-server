use std::{fmt::Debug, path::{Path, PathBuf}};
use sal_core::error::Error;
use crate::{kernel::EvalEx, server::{DeviceInfo, EvalResult, Query, Reply, Request, extract}};

///
/// Extracting incoming messages as [DeviceInfoRequest]
/// - Forwarding requested id to the specified `ctx`
/// - Returns [DeviceInfo]
pub(crate) struct SelectDevInfo {
    path: PathBuf,
}
//
//
impl SelectDevInfo {
    ///
    /// Returns [SortByX] new instance
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_owned(),
        }
    }
}
//
//
impl<K: Debug + Copy> EvalEx<Request<K>, EvalResult<K>> for SelectDevInfo {
    //
    fn eval(&self, req: Request<K>) -> EvalResult<K> {
        let error = Error::new("SelectDevInfo", "eval");
        let query = extract!(&req.query, Query::DeviceInfo)
            .map_err(|_| error.err(format!("Query::DeviceInfo expected, but found {:?}", req.query_id)))?;
        //
        // Do something woth incomong query...
        let dev_id = query.dev_id.clone();
        //
        // Generate and return reply to the request
        Ok(Some(req.reply(Reply::DeviceInfo(DeviceInfo {}))))
    }
    ///
    /// Halts hanbler
    fn exit(&self) {
        // Halt continuous operations here
    }
}
//
//
unsafe impl Send for SelectDevInfo {}
