use std::{fmt::Debug, path::{Path, PathBuf}};
use sal_core::error::Error;
use crate::{kernel::EvalEx, server::{DeviceDoc, EvalResult, Query, Reply, Request, extract}};

///
/// Extracting incoming messages as [DeviceDocRequest]
/// - Forwarding requested id to the specified `ctx`
/// - Returns [DeviceDoc]
pub struct SelectDevDoc {
    path: PathBuf,
}
//
//
impl SelectDevDoc {
    ///
    /// Returns [SelectDevDoc] new instance
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_owned(),
        }
    }
}
//
//
impl<K: Debug + Copy> EvalEx<Request<K>, EvalResult<K>> for SelectDevDoc {
    //
    fn eval(&self, req: Request<K>) -> EvalResult<K> {
        let error = Error::new("SelectDevDoc", "eval");
        let query = extract!(&req.query, Query::DeviceInfo)
            .map_err(|_| error.err(format!("Query::DeviceDoc expected, but found {:?}", req.query_id)))?;
                //
        // Do something woth incomong query...
        let _dev_id = query.dev_id.clone();
        //
        // Generate and return reply to the request
        Ok(Some(req.reply(Reply::DeviceDoc(DeviceDoc {}))))
    }
    ///
    /// Halts hanbler
    fn exit(&self) {
        // Halt continuous operations here
    }
}
//
//
unsafe impl Send for SelectDevDoc {}
