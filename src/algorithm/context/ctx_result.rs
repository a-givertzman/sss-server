use std::ops::{FromResidual, Try};

use bincode::{Decode, Encode};

///
/// Enum for structurizing types of result's
#[derive(Debug, Clone, PartialEq, Decode, Encode)]
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
//
//
impl<T, E> From <Result<T, E>> for CtxResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => CtxResult::Ok(v),
            Err(e) => CtxResult::Err(e),
        }
    }
}
//
//
impl<T, E> From <CtxResult<T, E>> for Result<T, sal_core::error::Error> 
where E: std::error::Error, sal_core::error::Error: From<E> {
    fn from(value: CtxResult<T, E>) -> Self {
        match value {
            CtxResult::Ok(v) => Result::Ok(v),
            CtxResult::Err(e) => Result::Err(sal_core::error::Error::new("CtxResult", "from").pass_with("{}", e)),
            CtxResult::None => Result::Err(sal_core::error::Error::new("CtxResult", "from").err("None")),
        }
    }
}
//
//
impl<T> std::ops::FromResidual for CtxResult<T, crate::Error> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(e) => CtxResult::Err(e),
        }
    }
}
//
//
impl<T, E> Try for CtxResult<T, E> {
    type Output = T;
    type Residual = Result<std::convert::Infallible, crate::Error>;

    fn from_output(output: Self::Output) -> Self {
        CtxResult::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            CtxResult::Ok(v) => std::ops::ControlFlow::Continue(v),
            CtxResult::Err(e) => std::ops::ControlFlow::Break(Err(e)),
            CtxResult::None => unimplemented!(),
        }
    }
}
