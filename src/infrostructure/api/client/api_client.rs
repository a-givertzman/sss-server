use api_tools::client::{api_query::{ApiQuery, ApiQueryKind, ApiQuerySql}, api_request::ApiRequest};

use crate::kernel::error::error::Error;

///
/// Provides access to the API Server
#[derive(Debug, Clone)]
pub struct ApiClient {
    database: String,
    host: String,
    port: String,
}
//
impl ApiClient {
    pub fn new(database: String, host: String, port: String) -> Self {
        Self {
            database,
            host,
            port,
        }
    }
    //
    pub fn fetch(&self, sql: &str) -> Result<Vec<u8>, Error> {
        let mut request = ApiRequest::new(
            &api_tools::debug::dbg_id::DbgId("parent".to_owned()),
            self.host.clone() + ":" + &self.port,
            "auth_token",
            ApiQuery::new(
                ApiQueryKind::Sql(ApiQuerySql::new(&self.database, sql)),
                false,
            ),
            true,
            false,
        );
        request
            .fetch(true)
            .map_err(|e| Error::FromString(format!("ApiServer fetch error: {e}")))
    }
}