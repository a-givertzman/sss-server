use nalgebra::*;
use obj::{Obj, ObjData};
use parry3d_f64::shape::{Cuboid, TriMesh, TriMeshFlags};
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Shape {
    dbg: Dbg,
    path: PathBuf,
    mesh: Option<TriMesh>,
    dx: f64,
    scale: f64,
}

unsafe impl Send for Shape {}

impl Shape {
    pub fn new(parent: &Dbg, path: PathBuf, dx: f64, scale: f64) -> Self {
        let dbg = Dbg::new(parent, "Shape");
        Self {
            dbg,
            path: path.into(),
            mesh: None,
            dx,
            scale,
        }
    }
    /// Init shape, load geometry
    pub fn init(&mut self) -> Result<(), Error> {
        if self.mesh.is_none() {
            let error = Error::new(&self.dbg, "init");
            let mesh =
                load(self.path.clone()).map_err(|err| error.pass_with("load", err.to_string()))?;
            let scale = 1. / self.scale;
            self.mesh = Some(mesh.scaled(&Vector3::new(scale, scale, scale)));
        }
        Ok(())
    }
    /// calculate displacement data
    /// result: [heel, trim, draught, volume, x, y, z]
    pub fn displacement(&self, heel: f64, trim: f64, draught: f64) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "intersect");
        let mesh = match self.intersect(heel, trim, draught) {
            Ok(mesh) => mesh,
            Err(err) => return Err(error.pass_with("self.intersect", err)),
        };
        let properties = parry3d_f64::shape::Shape::mass_properties(&mesh, 1.);
        let local_com = properties.local_com;
        Ok(vec![
            heel,
            trim,
            draught,
            1. / properties.inv_mass,
            local_com.x - self.dx,
            local_com.y,
            local_com.z,
        ])
    }
    /// intersect shape with water line
    fn intersect(&self, heel: f64, trim: f64, draught: f64) -> Result<TriMesh, Error> {
        let error = Error::new(&self.dbg, "intersect");
        let Some(mesh) = self.mesh.clone() else {
            dbg!("shape intersect error: no mesh", heel, trim, draught);
            return Err(error.err("self.mesh is none!"));
        };
        let heel_rad = heel.to_radians();
        let trim_rad = -trim.to_radians();
        let center = Translation3::new(self.dx, 0., draught);
        let cuboid_half_size = 1000.;
        let cuboid = Cuboid::new(Vector3::repeat(cuboid_half_size));
        let cuboid_rotation = UnitQuaternion::from_euler_angles(heel_rad, trim_rad, 0.);
        let point = Point3::new(0.0, 0.0, -cuboid_half_size);
        let point = (center * cuboid_rotation).transform_point(&point);
        let cuboid_translation = Translation3::new(point.x, point.y, point.z);
        let result = mesh.intersection_with_local_cuboid(
            false,
            &cuboid,
            &Isometry::from_parts(cuboid_translation, cuboid_rotation),
            false,
            0.0000001,
        );
        match result {
            Ok(mesh) => match mesh {
                Some(mesh) => return Ok(mesh),
                None => return Err(error.err(format!("mesh.intersection_with_local_cuboid: no mesh for heel:{heel}, trim:{trim}, draught:{draught}"))),
            },
            Err(err) => {
                return Err(error.pass_with(format!("mesh.intersection_with_local_cuboid heel:{heel}, trim:{trim}, draught:{draught}"), err.to_string()));
            }
        };
    }
}
// load data from .obj file
fn load(path: PathBuf) -> Result<TriMesh, Error> {
    let error = Error::new("Shape", "load");
    let Obj {
        data: ObjData {
            position, objects, ..
        },
        ..
    } = match Obj::load(path) {
        Ok(obj) => obj,
        Err(err) => return Err(error.pass_with("Obj::load(path)", err.to_string())),
    };
    let position = position
        .iter()
        .map(|v| Point3::new(v[0] as f64, v[1] as f64, v[2] as f64))
        .collect::<Vec<_>>();
    let objects = objects[0].groups[0]
        .polys
        .iter()
        .map(|p| [p.0[0].0 as u32, p.0[1].0 as u32, p.0[2].0 as u32])
        .collect::<Vec<_>>();
    match TriMesh::with_flags(position, objects, TriMeshFlags::all()) {
        Ok(mesh) => Ok(mesh),
        Err(err) => return Err(error.pass_with("TriMesh::with_flags", err.to_string())),
    }
}
