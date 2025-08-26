use nalgebra::*;
use obj::{Obj, ObjData};
use parry3d_f64::shape::{TriMesh, TriMeshFlags};
use sal_core::error::Error;
use std::io::Write;
use std::path::PathBuf;
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
    let (result, empty_normals): (Vec<_>, Vec<_>)  = mesh.triangles()
        .map(|t| (t.normal(), t) )
        .partition(|(n, _)| n.is_some());
    if !empty_normals.is_empty() {
        return Err(error.err(format!("calculate normal error, path:{:?}", path)));
    }
    let triangles: Vec<_> = result.iter()
        .map(|(n, t)| {
            let n = n.unwrap();
            let normal = stl_io::Vector([
                n[0] as f32,
                n[1] as f32,
                n[2] as f32,
            ]);
            let vertices = [
                stl_io::Vector([
                    t.a[0] as f32,
                    t.a[1] as f32,
                    t.a[2] as f32,
                ]),
                stl_io::Vector([
                    t.b[0] as f32,
                    t.b[1] as f32,
                    t.b[2] as f32,
                ]),
                stl_io::Vector([
                    t.c[0] as f32,
                    t.c[1] as f32,
                    t.c[2] as f32,
                ]),
            ];
            stl_io::Triangle {normal, vertices,} 
        })
        .collect();
    let mut binary_stl = Vec::<u8>::new();
    stl_io::write_stl(&mut binary_stl, triangles.iter())
        .map_err(|err| error.pass_with("stl_io::write_stl", err.to_string()))?;
    let mut buffer = std::fs::File::create(&path)
        .map_err(|err| error.pass_with(format!("File::create, path:{:?}", path), err.to_string()))?;
    buffer.write_all(&binary_stl)
        .map_err(|err| error.pass_with(format!("buffer.write_all, path:{:?}", path), err.to_string()))
}

