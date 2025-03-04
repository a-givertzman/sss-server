use crate::{algorithm::context::context::Context, kernel::sync::link::Link};
///
/// Used for declarative `Rrequest` implementation
/// 
/// Example:
/// ```ignore
/// let math = AlgoSecond::new(
///     req: Request<T>::new(op: |ctx: &Context, link: &Link| -> T {
///         // Query: Some Struct comtains all neccessary info and implements `Serialize`
///         let query = QueryStruct::new();
///         // Reply: Returns `T`, implements `Deserialize`
///         link.req(query)
///     }),
///     eval: AlgFirst::new(initial),
/// )
/// ```
pub struct Request<T> {
    op: Box<dyn Fn(&Context, Link) -> T + Send + Sync>,
}
//
//
impl<T> Request<T> {
    ///
    /// Returns [Request] new instance
    /// - `op` - the body of the request
    pub fn new(op: impl Fn(&Context, Link) -> T + Send + Sync + 'static) -> Self {
        Self { op: Box::new(op) }
    }
    ///
    /// Performs the request defined in the `op`
    pub fn fetch(&self, ctx: &Context, link: Link) -> T {
        (self.op)(ctx, link)
    }
}
