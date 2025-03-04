///
/// Enum for structurizing types of result's
#[derive(Debug, Clone, PartialEq)]
pub enum CtxResult<T, E> {
    /// positive type of result
    Ok(T),
    /// result type with error
    Err(E),
    /// empty result
    None,
}
//
//
impl<T, E> CtxResult<T, E> {
    //
    //
    #[allow(dead_code)]
    pub fn unwrap(self) -> T
    where
        E: std::fmt::Debug,
    {
        match self {
            CtxResult::Ok(t) => t,
            CtxResult::Err(err) => {
                panic!("called `Result::unwrap()` on an `Err` value, \n\t{:?}", err)
            }
            CtxResult::None => panic!("called `Result::unwrap()` on an `None` value"),
        }
    }
}
//
//
impl<T, E> Default for CtxResult<T, E> {
    fn default() -> Self {
        Self::None
    }
}
