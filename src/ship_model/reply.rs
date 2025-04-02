use sal_sync::services::entity::error::str_err::StrErr;
///
/// Replies from the `ShipModel`
#[derive(Debug)]
pub enum Reply {
    AreasStrength(Result<(Vec<f64>, Vec<f64>), StrErr>),
    ComputeBalance( TODO ),
}
