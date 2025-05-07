///
/// Defines relative position of a 3D object.
///
/// Considered to be used to filter out [super::ShipModel] elements.
/// See [super::ShipModel::subvolume] to find an example of use.
pub enum RelativePostion {
    Above,
    Under,
}
