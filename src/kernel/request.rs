use sal_sync::sync::Owner;
use crate::kernel::sync::Link;

///
/// Used for declarative `Rrequest` implementation
/// 
/// Example:
/// ```ignore
/// let math = AlgoSecond::new(
///     req: Request<T>::new(op: |ctx: Context, link: Link| -> T {
///         // Query: Some Struct comtains all neccessary info and implements `Serialize`
///         let query = QueryStruct::new();
///         // Reply: Returns `T`, implements `Deserialize`
///         let reply = link.req(query)
///         // Returning received reply and link
///         (reply, link)
///     }),
///     eval: AlgFirst::new(initial),
/// )
/// ```
pub struct Request<In, Out> {
    link: Owner<Link>,
    op: Box<dyn Fn(In, Link) -> (Out, Link)>,
}
//
//
impl<In, Out> Request<In, Out> {
    ///
    /// Returns [Request] new instance
    /// - `link` - `Link` - communication entity
    /// - `op` - the body of the request
    pub fn new(link: Link, op: impl Fn(In, Link) -> (Out, Link) + Send + Sync + 'static) -> Self {
        Self {
            link: Owner::new(link),
            op: Box::new(op),
        }
    }
    ///
    /// Performs the request defined in the `op`
    pub fn fetch(&self, val: In) -> Out {
        let link = self.link.take().unwrap();
        let (result, link) = (self.op)(val, link);
        self.link.replace(link);
        result
    }
}
