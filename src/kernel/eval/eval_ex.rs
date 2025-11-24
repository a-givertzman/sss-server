pub trait EvalEx<In, Out> {
    fn eval(&self, val: In) -> Out;
    fn exit(&self);
}
