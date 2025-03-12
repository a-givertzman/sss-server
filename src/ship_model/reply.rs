use sal_sync::services::entity::error::str_err::StrErr;
///
/// Replies from the `ShipModel`
#[derive(Debug)]
pub enum Reply {
    AreasStrength(Result<(Vec<crate::algorithm::entities::data::strength::VerticalArea>, Vec<crate::algorithm::entities::area::HAreaStrength>), StrErr>)
}
