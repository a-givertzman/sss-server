pub trait EvalEx<In, Out> {
    ///
    /// Perform an operation
    fn eval(&self, val: In) -> Out;
    ///
    /// Halt an operation
    fn exit(&self);
}
