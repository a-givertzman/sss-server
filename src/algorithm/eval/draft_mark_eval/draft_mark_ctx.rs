//! Расчет уровня заглубления для координат отметок заглубления на корпусе судна
use super::DraftMarkResult;

#[derive(Debug, Clone)]
pub struct DraftMarkCtx {
    pub data: Vec<DraftMarkResult> 
}
