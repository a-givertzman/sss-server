//! заглушка модели для тестирования
use sal_core::error::Error;
use sal_sync::services::entity::name::Name;
use crate::{algorithm::entities::*, ship_model::{model_link::IModelLink, query::*, reply::*}};
use super::data::*;
//
//
#[derive(Debug)]
pub struct FakeModelLink {
    txid: usize,
    name: Name,
    bounds: Bounds, // Bounds::from_min_max(-3.6, 135.5, 200).unwrap()
}
//
//
impl IModelLink for FakeModelLink {
    ///
    /// Type doc comment
    fn bounds(&self) -> Result<Bounds, Error> {
        Ok(self.bounds.clone())
    }
    ///
    /// - Returns areas by ship frames
    fn bound_areas(&self) -> Result<(Vec<f64>, Vec<f64>), Error> {
        let area_h_str: Vec<_> = area_h_str::area_h_str()
            .data()
            .into_iter()
            .map(|v| (v.value, Bound::new(v.bound_x1, v.bound_x2).unwrap()))
            .collect();
        let area_v_str: Vec<_> = area_v_str::area_v_str()
            .data()
            .into_iter()
            .map(|v| (v.value, Bound::new(v.bound_x1, v.bound_x2).unwrap()))
            .collect();
        let (area_v_str, area_h_str): (Vec<f64>, Vec<f64>) = self.bounds
            .iter()
            .map(|b1| {
                (
                    area_v_str.iter().fold(0., |sum, &(v, b2)| {
                        sum + v * b1.part_ratio(&b2).unwrap_or(0.)
                    }),
                    area_h_str.iter().fold(0., |sum, &(v, b2)| {
                        sum + v * b1.part_ratio(&b2).unwrap_or(0.)
                    }),
                )
            })
            .collect();
        Ok((area_v_str, area_h_str))
    }
    ///
    /// - Returns areas by ship frames
    fn compute_balance(&self, data: BalanceSrcData) -> Result<BalanceResultData, Error> {
        todo!()
    }
}
