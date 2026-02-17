use sal_core::error::Error;
use crate::server::Response;

pub type EvalResult<QueryId> = Result<Option<Response<QueryId>>, Error>;