use futures::future::BoxFuture;
///
/// Trate defines common evaluation function for calculations classes
pub trait Eval<Inp, Out> {
    ///
    /// Performs a calculation
    /// - Returns [Out] contains results inside
    fn eval(&mut self, val: Inp) -> BoxFuture<'_, Out>;
}
