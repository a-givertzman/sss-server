use nalgebra::*;
use parry3d_f64::shape::{Cuboid, TriMesh};
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use std::path::PathBuf;

use crate::algorithm::entities::Position;
use crate::algorithm::entities::model_cached::local_cache::shape::compartment_center;
use crate::algorithm::entities::model_cached::{Shape, load_stl};

#[derive(Clone)]
pub struct DisplacementShape {
    dbg: Dbg,
    mesh: Option<TriMesh>,
    path: Option<PathBuf>,
    center: Option<Point3<f64>>,
    scale: f64,
    epsilon: f64,
    resolution: u32,
}

unsafe impl Send for DisplacementShape {}

impl DisplacementShape {
    /// Конструктор
    /// * parent - Dbg родителя
    /// * mesh - модель
    /// * path - путь к файлу, содержащему модель
    /// * dx - смещение миделя относительно центра координат модели
    /// * scale - масштаб модели для ее приведения к метрам (1000: модель в мм)
    /// * epsilon - точность расчета сечений
    /// * resolution - точность расчета момента инерции
    pub fn new(
        parent: &Dbg,
        mesh: Option<TriMesh>,
        path: Option<PathBuf>,
        center: Option<Point3<f64>>,
        scale: f64,
        epsilon: f64,
        resolution: u32,
    ) -> Self {
        let dbg = Dbg::new(parent, "DisplacementShape");
        Self {
            dbg,
            mesh,
            path,
            center,
            scale,
            epsilon,
            resolution,
        }
    }
    /// Конструктор для создания "ленивого" экземпляра.
    /// После создания обязателен вызов метода "init".
    /// center - смещение центра координат для расчетов относительно центра координат меша, 
    /// для отсеков задается как None и считается автоматом
    pub fn new_uninit(
        parent: &Dbg,
        path: PathBuf,
        center: Option<Position>,
        scale: f64,
    ) -> Self {
        Self::new(
            parent,
            None,
            Some(path),
            center.map(|p| Point3::new(p.x(), p.y(), p.z())),
            scale,
            0.0000001,
            10000,
        )
    }
    /// Init shape, load geometry
    pub fn init(&mut self) -> Result<(), Error> {
        if self.mesh.is_none() {
            let error = Error::new(&self.dbg, "init");
            let mut mesh = load_stl(&self.path.clone().ok_or(error.err("empty path"))?)
                .map_err(|err| error.pass_with("load", err.to_string()))?;
            let scale = 1. / self.scale;
            mesh = mesh.scaled(&Vector3::new(scale, scale, scale));
            if self.center.is_none() {
                self.center = Some(compartment_center(&mesh));
            }
            self.mesh = Some(mesh);
        }
        Ok(())
    }
    ///
    /// Расчет водоизмещения судна и положение его центра в связанной с судной системой координат
    /// result: [volume, x, y, z]
    pub fn displacement(
        &self,
        heel: f64,
        trim: f64,
        draught: f64,
    ) -> Result<(f64, f64, f64, f64), Error> {
        let error = Error::new(&self.dbg, "displacement");
        let position = self
            .position(heel, trim, draught)
            .map_err(|err| error.pass_with("self.position", err))?;
        let cuboid_half_size = 1000.;
        let cuboid = Cuboid::new(Vector3::repeat(cuboid_half_size));
        let result = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .intersection_with_cuboid(
                &position,
                false,
                &cuboid,
                &Isometry::from_parts(
                    Translation3::new(0., 0., -cuboid_half_size),
                    UnitQuaternion::identity(),
                ),
                false,
                self.epsilon,
            );
        let mesh = match result {
            Ok(mesh) => match mesh {
                Some(mesh) => mesh,
                None => {
                    let center = self.center.unwrap();
                    return Ok((0., center.x, center.y, center.z + draught));
                } // return Err(error.err("mesh.intersection_with_plane error: no intersection!"));
            },
            Err(e) => return Err(error.pass_with("mesh.intersection_with_plane", e.to_string())),
        };
        let properties = parry3d_f64::shape::Shape::mass_properties(&mesh, 1.);
        Ok((
            1. / properties.inv_mass,
            properties.local_com.x,
            properties.local_com.y,
            properties.local_com.z,
        ))
    }
    ///
    /// Расчет площади ватерлинии судна и положение ее центра в связанной с судной системой координат
    /// result: [area, x, y, z]
    pub fn waterline_area(
        &self,
        heel: f64,
        trim: f64,
        draught: f64,
    ) -> Result<(f64, f64, f64, f64), Error> {
        let error = Error::new(&self.dbg, "area");
        let position = self
            .position(heel, trim, draught)
            .map_err(|err| error.pass_with("self.position", err))?;
        let cuboid_half_size = 1000.;
        let hdz = 0.005;
        let cuboid = Cuboid::new(Vector3::new(cuboid_half_size, cuboid_half_size, hdz));
        let result = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .intersection_with_cuboid(
                &position,
                false,
                &cuboid,
                &Isometry::identity(),
                false,
                self.epsilon,
            );
        let mesh = match result {
            Ok(mesh) => match mesh {
                Some(mesh) => mesh,
                None => {
                    let center = self.center.unwrap();
                    return Ok((0., center.x, center.y, center.z + draught));
                } //  return Err(error.err("mesh.intersection_with_plane error: no intersection!"));
            },
            Err(e) => return Err(error.pass_with("mesh.intersection_with_plane", e.to_string())),
        };
        let properties = parry3d_f64::shape::Shape::mass_properties(&mesh, 0.5 / hdz);
        Ok((
            1. / properties.inv_mass,
            properties.local_com.x,
            properties.local_com.y,
            properties.local_com.z,
        ))
    }
    ///
    /// Расчет [длинны и ширины по ватерлинии](https://github.com/a-givertzman/sss/blob/6d91fb09de073995c3a165ebaaa76e4f1e202f36/design/algorithm/part04_stability/chapter05_criteria/section02_weatherCriteria.md)
    pub fn aabb(&self, draught: f64) -> Result<(f64, f64), Error> {
        let error = Error::new(&self.dbg, "aabb");
        let result = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .intersection_with_plane(
                &Isometry::identity(),
                &Vector3::z_axis(),
                draught,
                self.epsilon,
            );
        return match result {
            parry3d_f64::query::IntersectResult::Intersect(polyline) => {
                let vertices: Vec<_> = polyline
                    .vertices()
                    .iter()
                    .map(|p| Point2::new(p.x, p.y))
                    .collect();
                let (vx, vy): (Vec<_>, Vec<_>) = vertices.iter().map(|p| (p.x, p.y)).unzip();
                let min_x = vx
                    .iter()
                    .min_by(|&a, &b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let max_x = vx
                    .iter()
                    .max_by(|&a, &b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let min_y = vy
                    .iter()
                    .min_by(|&a, &b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let max_y = vy
                    .iter()
                    .max_by(|&a, &b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let dx = max_x - min_x;
                let dy = max_y - min_y;
                Ok((dx, dy))
            }
            parry3d_f64::query::IntersectResult::Negative => Ok((0., 0.)),
            parry3d_f64::query::IntersectResult::Positive => Ok((0., 0.)),
        };
    }
    ///
    /// Расчет [момента инерции свободной поверхности жидкости](https://github.com/a-givertzman/sss/blob/cdef1e9a2133adeb2fe8abcda6229b206c28493c/design/algorithm/part04_stability/chapter01_initialStability/chapter01_initialStability.md#%D0%B2%D0%BB%D0%B8%D1%8F%D0%BD%D0%B8%D0%B5-%D1%81%D0%B2%D0%BE%D0%B1%D0%BE%D0%B4%D0%BD%D0%BE%D0%B9-%D0%BF%D0%BE%D0%B2%D0%B5%D1%80%D1%85%D0%BD%D0%BE%D1%81%D1%82%D0%B8)
    pub fn inertia(&self, heel: f64, trim: f64, draught: f64) -> Result<(f64, f64), Error> {
        let error = Error::new(&self.dbg, "inertia");
        let position = self
            .position(heel, trim, draught)
            .map_err(|err| error.pass_with("self.position", err))?;
        let result = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .intersection_with_plane(&position, &Vector3::z_axis(), 0., self.epsilon);
        match result {
            parry3d_f64::query::IntersectResult::Intersect(polyline) => {
                let vertices: Vec<_> = polyline
                    .vertices()
                    .iter()
                    .map(|p| position.transform_point(&p))
                    .map(|p| Point2::new(p.x, p.y))
                    .collect();
                let indices = polyline.indices().to_owned();
                let mut voxel_set = parry2d_f64::transformation::voxelization::VoxelSet::voxelize(
                    &vertices,
                    &indices,
                    self.resolution,
                    parry2d_f64::transformation::voxelization::FillMode::FloodFill {
                        detect_cavities: true,
                        detect_self_intersections: false,
                    },
                    false,
                );
                let scale = voxel_set.scale;
                let qrt_scale = scale * scale;
                let voxels_volume = voxel_set.compute_volume();
                let voxel_volume = voxel_set.voxel_volume();
                let (v_x, v_y) = voxel_set
                    .voxels()
                    .iter()
                    .fold((0., 0.), |(v_x, v_y), voxel| {
                        (v_x + voxel.coords.x as f64, v_y + voxel.coords.y as f64)
                    });
                let voxel_area_center_x = v_x * voxel_volume / voxels_volume;
                let voxel_area_center_y = v_y * voxel_volume / voxels_volume;
                voxel_set.compute_bb();
                let max_bb = voxel_set.max_bb_voxels();
                let x_array: Vec<_> = (0..=max_bb.x)
                    .map(|v| v as f64 - voxel_area_center_x)
                    .map(|v| v * v)
                    .collect();
                let y_array: Vec<_> = (0..=max_bb.y)
                    .map(|v| v as f64 - voxel_area_center_y)
                    .map(|v| v * v)
                    .collect();
                let (i_x, i_y) = voxel_set
                    .voxels()
                    .iter()
                    .fold((0., 0.), |(i_x, i_y), voxel| {
                        (
                            i_x + y_array[voxel.coords.y as usize].clone(),
                            i_y + x_array[voxel.coords.x as usize],
                        )
                    });
                let i_x = i_x * qrt_scale * voxel_volume;
                let i_y = i_y * qrt_scale * voxel_volume;
                Ok((i_x, i_y))
            }
            parry3d_f64::query::IntersectResult::Negative => Ok((0., 0.)),
            parry3d_f64::query::IntersectResult::Positive => Ok((0., 0.)),
        }
    }
}

impl Shape for DisplacementShape {
    /// Init shape, load geometry
    fn init(&mut self) -> Result<(), Error> {
        if self.mesh.is_none() {
            let error = Error::new(&self.dbg, "init");
            let mut mesh = load_stl(&self.path.clone().ok_or(error.err("empty path"))?)
                .map_err(|err| error.pass_with("load", err.to_string()))?;
            let scale = 1. / self.scale;
            mesh = mesh.scaled(&Vector3::new(scale, scale, scale));
            if self.center.is_none() {
                self.center = Some(compartment_center(&mesh));
            }
            self.mesh = Some(mesh);
        }
        Ok(())
    }
    //
    fn dbg(&self) -> &Dbg {
        &self.dbg
    }
    //
    fn mesh(&self) -> Option<&TriMesh> {
        self.mesh.as_ref()
    }
    //
    fn center(&self) -> Option<&Point3<f64>> {
        self.center.as_ref()
    }
}
