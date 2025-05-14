use api_tools::client::{api_query::{ApiQuery, ApiQueryKind, ApiQuerySql}, api_request::ApiRequest};
use sal_core::{dbg::Dbg, error::Error};

///
/// Provides access to the API Server
#[derive(Debug, Clone)]
pub struct ApiClient {
    dbg: Dbg,
    database: String,
    host: String,
    port: String,
}
//
impl ApiClient {
    pub fn new(parent: impl Into<String>, database: String, host: String, port: String) -> Self {
        Self {
            dbg: Dbg::new(parent, "ApiClient"),
            database,
            host,
            port,
        }
    }
    ///
    /// Performs an API request with the parameters specified in the constructor
    pub fn fetch(&self, sql: &str) -> Result<Vec<u8>, Error> {
        let mut request = ApiRequest::new(
            &self.dbg,
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
            .map_err(|e| Error::new("ApiClient", "fetch").pass(e.to_string()))
    }
}