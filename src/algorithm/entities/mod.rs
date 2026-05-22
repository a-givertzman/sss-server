pub mod icing_coeff;
pub mod data;
pub mod model_cached;
pub mod ship_model;
pub mod recalculation_course_angular;
pub mod draught;

pub use sal_3dlib_core::cache::Cache as Cache;
use sal_core::{dbg::Dbg, error::Error};
use crate::prelude::{Context, InitialCtx};



