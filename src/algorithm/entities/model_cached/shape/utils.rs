use nalgebra::*;
use obj::{Obj, ObjData};
use parry3d_f64::shape::{TriMesh, TriMeshFlags, Triangle};
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
/// Быстрый расчет по треугольникам
pub fn triangle_submerged_volume(tri: &Triangle, z_w: f64) -> f64 {
    let p = [tri.a, tri.b, tri.c];
    let mut poly = [Point3::origin(); 4]; // Максимум 4 точки при сечении треугольника плоскостью
    let mut n = 0;

    for i in 0..3 {
        let a = p[i];
        let b = p[(i + 1) % 3];

        if a.z <= z_w {
            poly[n] = a;
            n += 1;
        }

        if (a.z < z_w && b.z > z_w) || (a.z > z_w && b.z < z_w) {
            let t = (z_w - a.z) / (b.z - a.z);
            poly[n] = a + (b - a) * t;
            n += 1;
        }
    }

    if n < 3 { return 0.0; }

    // Интегрирование по вееру (Fan) относительно (0,0,0)
    let mut v = 0.0;
    let v0 = poly[0].coords;
    for i in 1..n - 1 {
        v += v0.cross(&poly[i].coords).dot(&poly[i+1].coords);
    }
    v / 6.0
}
/// Генерирует треугольники-заглушки для сечения меша плоскостью X = x_limit
pub fn generate_cap_triangles(
    mesh_indices: &[[u32; 3]],
    vertices: &[Point3<f64>],
    indices_in_bound: &[u32],
    x_limit: f64,
    look_outside_positive: bool, // true если нормаль должна смотреть в +X
) -> Vec<Triangle> {
    let mut segments = Vec::new();

    // 1. Ищем ребра, пересекающие плоскость x_limit
    for &tri_idx in indices_in_bound {
        let tri_nodes = mesh_indices[tri_idx as usize];
        let v = [
            vertices[tri_nodes[0] as usize],
            vertices[tri_nodes[1] as usize],
            vertices[tri_nodes[2] as usize],
        ];

        for i in 0..3 {
            let v1 = v[i];
            let v2 = v[(i + 1) % 3];

            // Проверяем, пересекает ли ребро плоскость X
            if (v1.x < x_limit && v2.x > x_limit) || (v1.x > x_limit && v2.x < x_limit) {
                let t = (x_limit - v1.x) / (v2.x - v1.x);
                let intersect = v1 + (v2 - v1) * t;
                segments.push(intersect);
            }
        }
    }

    if segments.len() < 3 { return vec![]; }

    // 2. Простая триангуляция через "веер" относительно центра сечения
    // Для сложных вогнутых сечений (L-образные корпуса) лучше использовать earcut,
    // но для судна центроид обычно работает.
    let center_coords = segments.iter().map(|p| p.coords).sum::<Vector3<f64>>() / segments.len() as f64;
    let center = Point3::from(center_coords);

    // Сортируем точки по углу в плоскости YZ, чтобы собрать правильный контур
    segments.sort_by(|a, b| {
        let ang_a = (a.y - center.y).atan2(a.z - center.z);
        let ang_b = (b.y - center.y).atan2(b.z - center.z);
        ang_a.partial_cmp(&ang_b).unwrap()
    });

    let mut cap_tris = Vec::new();
    for i in 0..segments.len() {
        let p1 = segments[i];
        let p2 = segments[(i + 1) % segments.len()];
        
        // Создаем треугольник и проверяем его нормаль
        let mut tri = Triangle::new(center, p1, p2);
        let normal = (p1 - center).cross(&(p2 - center));
        
        // Направляем нормаль наружу отсека по оси X
        let needs_flip = if look_outside_positive { normal.x < 0.0 } else { normal.x > 0.0 };
        if needs_flip {
            tri = Triangle::new(center, p2, p1);
        }
        cap_tris.push(tri);
    }
    cap_tris
}