///
/// Trate defines common evaluation function for calculations classes
pub trait EvalMut<Inp, Out> {
    ///
    /// Performs a calculation
    /// - Returns [Out] contains results inside
    fn eval(&self, val: Inp) -> Out;
}
