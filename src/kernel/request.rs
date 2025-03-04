use coco::Stack;
use futures::future::BoxFuture;
use super::sync::link::Link;
///
/// Used for declarative `Rrequest` implementation
/// 
/// Example:
/// ```ignore
/// let math = AlgoSecond::new(
///     link: switch.link(),
///     req: Request<In, Out>::new(async |val: In, link: Link| -> Out {
///         // Query: Some Struct comtains all neccessary info and implements `Serialize`
///         let query = QueryStruct::new();
///         // Reply: Returns `T`, implements `Deserialize`
///         (link.req(query).await, link)
///     }),
///     eval: AlgFirst::new(initial),
/// )
/// ```
pub struct Request<In, T> {
    link: Stack<Link>,
    op: Box<dyn AsyncFn<In, T> + Send + Sync>,
}
//
//
impl<In, T> Request<In, T> {
    ///
    /// Returns [Request] new instance
    /// - `link` - `Link` - communication entity
    /// - `op` - the body of the request
    pub fn new(link: Link, op: impl AsyncFn<In, T> + Send + Sync + 'static) -> Self {
        let stack = Stack::new();
        stack.push(link);
        Self {
            link: stack,
            op: Box::new(op),
        }
    }
    ///
    /// Performs the request defined in the `op`
    pub async fn fetch(&self, val: In) -> T {
        let link = self.link.pop().unwrap();
        let (result, link) = self.op.eval(val, link).await;
        self.link.push(link);
        result
    }
}
///
/// Async callback closure
pub trait AsyncFn<In, Out> {
    fn eval(&self, ctx: In, link: Link) -> BoxFuture<'_, (Out, Link)>;
}
//
//
impl<T, F, In, Out> AsyncFn<In, Out> for T
where
    T: Fn(In, Link) -> F,
    F: std::future::Future<Output = (Out, Link)> + Send + 'static,
{
    fn eval(&self, val: In, link: Link) -> BoxFuture<'_, (Out, Link)> {
        Box::pin(self(val, link))
    }
}
