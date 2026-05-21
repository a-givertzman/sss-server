use nalgebra::*;
use obj::{
    Obj, 
    ObjData
};
use parry3d_f64::math::{
    Isometry, 
    Vector, 
    Real
};
use parry3d_f64::shape::{
    Shape, 
    TriMesh, 
    TriMeshFlags
};
use sal_core::error::Error;
use std::io::Write;
use std::path::PathBuf;
use crate::algorithm::entities::Position;
///
/// Load data from .obj file
fn load_obj(path: PathBuf) -> Result<TriMesh, Error> {
    let error = Error::new("Shape", "load_obj");
    let Obj {
        data: ObjData {
            position, objects, ..
        },
        ..
    } = match Obj::load(path) {
        Ok(obj) => obj,
        Err(err) => return Err(error.pass_with("Obj::load(path)", err.to_string())),
    };
    let vertices = position
        .iter()
        .map(|v| Point3::new(v[0] as f64, v[1] as f64, v[2] as f64))
        .collect::<Vec<_>>();
    let indices = objects[0].groups[0]
        .polys
        .iter()
        .map(|p| [p.0[0].0 as u32, p.0[1].0 as u32, p.0[2].0 as u32])
        .collect::<Vec<_>>();
    TriMesh::with_flags(vertices, indices, TriMeshFlags::all())
        .map_err(|err| error.pass_with("TriMesh::with_flags", err.to_string()))
}
///
/// Load data from .stl file
pub fn load_stl(path: &PathBuf) -> Result<TriMesh, Error> {
    let error = Error::new("Shape", "load_stl");
    let file =
        std::fs::File::open(path).map_err(|err| error.pass_with("File::open", err.to_string()))?;
    let mut reader = std::io::BufReader::new(file);
    let stl_mesh = stl_io::read_stl(&mut reader)
        .map_err(|err| error.pass_with("stl_io::read_stl", err.to_string()))?;
    let vertices = stl_mesh
        .vertices
        .into_iter()
        .map(|v| Point3::new(v[0] as f64, v[1] as f64, v[2] as f64))
        .collect::<Vec<_>>();
    let indices = stl_mesh
        .faces
        .into_iter()
        .map(|f| {
            [
                f.vertices[0] as u32,
                f.vertices[1] as u32,
                f.vertices[2] as u32,
            ]
        })
        .collect::<Vec<_>>();
    TriMesh::with_flags(vertices, indices, TriMeshFlags::all())
        .map_err(|err| error.pass_with("TriMesh::with_flags", err.to_string()))
}
///
/// Write data to .stl file
pub fn write_stl(path: &PathBuf, mesh: &TriMesh) -> Result<(), Error> {
    let error = Error::new("Shape", "write_stl");
    let (result, empty_normals): (Vec<_>, Vec<_>) = mesh
        .triangles()
        .map(|t| (t.normal(), t))
        .partition(|(n, _)| n.is_some());
    if !empty_normals.is_empty() {
        return Err(error.err(format!("calculate normal error, path:{:?}", path)));
    }
    let triangles: Vec<_> = result
        .into_iter()
        .map(|(n, t)| {
            let n = n.unwrap();
            let normal = stl_io::Vector([n[0] as f32, n[1] as f32, n[2] as f32]);
            let vertices = [
                stl_io::Vector([t.a[0] as f32, t.a[1] as f32, t.a[2] as f32]),
                stl_io::Vector([t.b[0] as f32, t.b[1] as f32, t.b[2] as f32]),
                stl_io::Vector([t.c[0] as f32, t.c[1] as f32, t.c[2] as f32]),
            ];
            stl_io::Triangle { normal, vertices }
        })
        .collect();
    let mut binary_stl = Vec::<u8>::new();
    stl_io::write_stl(&mut binary_stl, triangles.iter())
        .map_err(|err| error.pass_with("stl_io::write_stl", err.to_string()))?;
    let mut buffer = std::fs::File::create(&path).map_err(|err| {
        error.pass_with(format!("File::create, path:{:?}", path), err.to_string())
    })?;
    buffer.write_all(&binary_stl).map_err(|err| {
        error.pass_with(
            format!("buffer.write_all, path:{:?}", path),
            err.to_string(),
        )
    })
}
/// полный объем модели
pub fn properties(mesh: &TriMesh, density: f64) -> (f64, Position) {
    let properties = parry3d_f64::shape::Shape::mass_properties(mesh, density);
    let mass = if properties.inv_mass > 0. { 1. / properties.inv_mass } else { 0. };
    (
        mass,
        Position::new(
            properties.local_com.x,
            -properties.local_com.y,
            properties.local_com.z,
        ),
    )
}
/// Объем меша
pub fn volume(mesh: &TriMesh) -> f64 {
    let inv_mass = parry3d_f64::shape::Shape::mass_properties(mesh, 1.).inv_mass;
    if inv_mass > 0. { 1. / inv_mass } else { 0. }
}
/// Поворот модели
pub fn rotate(mesh: &TriMesh, angle_x: Real, angle_y: Real, angle_z: Real) -> TriMesh {
    let mut tank_rotated = mesh.clone();
    let aabb = tank_rotated.compute_local_aabb();
    let center = aabb.center();
    let rotation = UnitQuaternion::from_axis_angle(&Vector::x_axis(), angle_x) * 
                   UnitQuaternion::from_axis_angle(&Vector::z_axis(), angle_z) * 
                   UnitQuaternion::from_axis_angle(&Vector::y_axis(), angle_y);
    let translation_to_origin = Isometry::translation(-center.x, -center.y, -center.z);
    let translation_back = Isometry::translation(center.x, center.y, center.z);
    let rotate_iso = Isometry::from_parts(Vector::zeros().into(), rotation);
    let transform = translation_back * rotate_iso * translation_to_origin;
    tank_rotated.transform_vertices(&transform);
    tank_rotated
}
/// Площадь меша
pub fn square(mesh: &TriMesh) -> f64 {
    let vertices = mesh.vertices();
    let indices = mesh.indices();
    indices.iter().map(|tri| {
        let v0 = vertices[tri[0] as usize];
        let v1 = vertices[tri[1] as usize];
        let v2 = vertices[tri[2] as usize];
        let side1 = v1 - v0;
        let side2 = v2 - v0;
        0.5 * side1.cross(&side2).norm()
    }).sum()
}