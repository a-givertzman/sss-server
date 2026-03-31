use log::Log;
use nalgebra::*;
use parry3d_f64::shape::{Cuboid, TriMesh, TriMeshFlags};
use parry3d_f64::bounding_volume::aabb::*;
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use crate::algorithm::entities::model_cached::shape::utils;
use crate::algorithm::entities::model_cached::{BoundShape, Shape, compartment_center, load_stl};
use crate::algorithm::entities::{Bound, Position};
use crate::kernel::types::Arc;

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
    /// * center - смещение относительно центра координат модели
    /// * scale - масштаб модели для ее приведения к метрам (1000: модель в мм)
    /// * epsilon - точность расчета сечений
    /// * resolution - точность расчета момента инерции
    pub fn new(
        parent: &Dbg,
        mesh: Option<TriMesh>,
        path: Option<PathBuf>,
        model_x: Option<f64>,
        scale: f64,
        epsilon: f64,
        resolution: u32,
    ) -> Self {
        let dbg = Dbg::new(parent, "DisplacementShape");
        Self {
            dbg,
            mesh,
            path,
            center: model_x.map(|x| Point3::new(x, 0., 0.)),
            scale,
            epsilon,
            resolution,
        }
    }
    /// Конструктор для создания "ленивого" экземпляра.
    /// После создания обязателен вызов метода "init".
    /// center - смещение центра координат для расчетов относительно центра координат меша,
    /// для отсеков задается как None и считается автоматом
    /// для поврежденных отсеков задается как для корпса судна
    pub fn new_uninit(parent: &Dbg, path: PathBuf, model_x: Option<f64>, scale: f64) -> Self {
        Self::new(parent, None, Some(path), model_x, scale, 0.0001, 10000)
    }
    /// часть меша, пападающая в bound
    pub fn part(&self, bound: &Bound) -> Result<Option<BoundShape>, Error> {
        let error = Error::new(&self.dbg, "part");
        let half_size_x = bound.length().ok_or(error.err("no bound.length"))? / 2.;
        let position_x = bound.center().ok_or(error.err("no bound.center"))?;

        let src_mesh = self.mesh.as_ref().ok_or(error.err("no mesh"))?;

        // AABB расширен по Y и Z, но строго ограничен по X
        let aabb = Aabb::new(
            Point3::new(position_x - half_size_x, -1e8, -1e8),
            Point3::new(position_x + half_size_x, 1e8, 1e8)
        );

        let indices: Vec<u32> = src_mesh.bvh().intersect_aabb(&aabb).collect();

        if indices.is_empty() {
            return Ok(None);
        }

        let mesh_indices = src_mesh.indices();
        let vertices = src_mesh.vertices();
        let mut is_flipped = Vec::with_capacity(indices.len());

        for &tri_idx in &indices {
            let tri = mesh_indices[tri_idx as usize];
            // Безопасное извлечение вершин
            let v0 = vertices[tri[0] as usize];
            let v1 = vertices[tri[1] as usize];
            let v2 = vertices[tri[2] as usize];

            let normal_x = (v1.y - v0.y) * (v2.z - v0.z) - (v1.z - v0.z) * (v2.y - v0.y);
            let tri_center_x = (v0.x + v1.x + v2.x) / 3.0;
            let dir_from_center = tri_center_x - position_x;

            // Если нормаль и вектор от центра сегмента смотрят в разные стороны — флипаем
            is_flipped.push(normal_x * dir_from_center < 0.0);
        }

        // 4. Передаем все данные в BoundShape
        Ok(Some(BoundShape::new(
            &self.dbg,
            &Arc::new(self.mesh.clone()), // Убедитесь, что self.mesh это Option<TriMesh>
            indices,
            is_flipped, // Добавьте это поле в конструктор BoundShape
            position_x,
            self.epsilon,
        )))
    }
    ///
    /// Расчет водоизмещения судна и положение его центра в связанной с судной системой координат
    /// result: [volume, x, y, z]
    pub fn displacement(
        &self,
        heel: f64,
        trim: f64,
        draught: f64,
    ) -> Result<(f64, Position), Error> {
        let error = Error::new(&self.dbg, "displacement");
        // println!("{}.displacement | start {:3} {:3} {:3}", &self.dbg, heel, trim, draught);
        let position = self
            .position(heel, trim, draught)
            .map_err(|err| error.pass_with("self.position", err))?;
        //let cuboid_half_size = 1000.;
        // let cuboid = Cuboid::new(Vector3::repeat(cuboid_half_size));
        //    let mesh = self.mesh.as_ref().ok_or(error.err("no mesh"))?;
        // println!("{}.displacement | intersection_with_cuboid {:3} {:3} {:3}", &self.dbg, heel, trim, draught);
        let mut src_mesh = self.mesh.as_ref().ok_or(error.err("no mesh"))?;
        let src_aabb = src_mesh.aabb(&Isometry::identity());
        let mut mesh;
        let mut epsilon = self.epsilon;
        loop {
            // TODO костыль для фикса бага: иногда меш обрезается криво
            // Тут проверяется что баунд вырезанного меша не превышает исходный меш
            let result = src_mesh.split(&position, &Vector::z_axis(), 0., self.epsilon);
            mesh = match result {
                parry3d_f64::query::SplitResult::Pair(mut mesh, _) => {
                    if let Err(error) = mesh
                        .set_flags(TriMeshFlags::all())
                        .map_err(|err| error.pass_with("mesh.set_flags", err.to_string()))
                    {
                        log::error!("{}", error);
                    }
                    //    let filename = format!("10_{:.3}.stl", draught);
                    //    let cache_dir: PathBuf = ("src/algorithm/entities/model_cached/test/sofia/disp_bounded/195/stl/".to_owned() + &filename).into();
                    //    super::write_stl(&cache_dir, &mesh);
                    mesh
                }
                parry3d_f64::query::SplitResult::Negative => {
                    return Ok(super::properties(&src_mesh, 1.));
                }
                parry3d_f64::query::SplitResult::Positive => {
                    let center = self.center.ok_or(error.err("no center"))?;
                    return Ok((0., Position::new(center.x, -center.y, center.z + draught)));
                }
            };
            let aabb = mesh.aabb(&Isometry::identity());
            if aabb.mins.x + epsilon < src_aabb.mins.x
                || aabb.maxs.x - epsilon > src_aabb.maxs.x
                || aabb.mins.y + epsilon < src_aabb.mins.y
                || aabb.maxs.y - epsilon > src_aabb.maxs.y
                || aabb.mins.z + epsilon < src_aabb.mins.z
                || aabb.maxs.z - epsilon > src_aabb.maxs.z
            {
                let error = format!(
                    "{} part error: wrong aabb, rebuild! epsilon:{epsilon} src_aabb:{:?} res_aabb:{:?}",
                    self.dbg, src_aabb, aabb
                );
                log::warn!("{error}");
                src_mesh = &mesh;
                epsilon = epsilon * 2.;
                continue;
            }
            break;
        }
        //     println!("{}.displacement | set_flags {:3} {:3} {:3}", &self.dbg, heel, trim, draught);
        if let Err(error) = mesh
            .set_flags(TriMeshFlags::all())
            .map_err(|err| error.pass_with("mesh.set_flags", err.to_string()))
        {
            log::error!("{}", error);
        }
        let properties = super::properties(&mesh, 1.);
        //    println!("{}.displacement | mass_properties {:3} {:3} {:3} {:3} {:3} {:3} {:3} {:3}", &self.dbg, heel, trim, draught, position.translation.x, position.translation.y, position.translation.z, properties.0, properties.1);
        Ok(properties)
    }
    ///
    /// Расчет площади ватерлинии судна и положение ее центра в связанной с судной системой координат
    /// result: [area, x, y, z]
    pub fn waterline_area(
        &self,
        heel: f64,
        trim: f64,
        draught: f64,
    ) -> Result<(f64, Position), Error> {
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
        let mut mesh = match result {
            Ok(mesh) => match mesh {
                Some(mesh) => mesh,
                None => {
                    let center = self.center.ok_or(error.err("no center"))?;
                    return Ok((0., Position::new(center.x, -center.y, center.z + draught)));
                } //  return Err(error.err("mesh.intersection_with_cuboid error: no intersection!"));
            },
            Err(e) => return Err(error.pass_with("mesh.intersection_with_cuboid", e.to_string())),
        };
        if let Err(error) = mesh
            .set_flags(TriMeshFlags::all())
            .map_err(|err| error.pass_with("mesh.set_flags", err.to_string()))
        {
            log::error!("{}", error);
        }
        Ok(super::properties(&mesh, 0.5 / hdz))
    }
    ///
    /// Полный размер модели (длинна, ширина, высота, минимальная высота)
    pub fn size(&self) -> Result<(f64, f64, f64, f64), Error> {
        let error = Error::new(&self.dbg, "size");
        let aabb = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .aabb(&Isometry::identity());
        Ok((
            (aabb.maxs.x - aabb.mins.x),
            (aabb.maxs.y - aabb.mins.y),
            (aabb.maxs.z - aabb.mins.z),
            aabb.mins.z,
        ))
    }
    /// полный объем модели
    pub fn properties(&self) -> Result<(f64, Position), Error> {
        let error = Error::new(&self.dbg, "volume");
        Ok(super::properties(
            self.mesh.as_ref().ok_or(error.err("no mesh"))?,
            1.,
        ))
    }
    ///
    /// Расчет водоизмещения для разных осадок (для шпации)
    pub fn displacement_by_steps(&self, step: f64) -> Result<Vec<(f64, f64)>, Error> {
        let error = Error::new(&self.dbg, "displacement_by_steps");
        let aabb = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .aabb(&Isometry::identity());
        let draught_min = aabb.mins.z;
        let draught_max = aabb.maxs.z;
        let mut steps = vec![(-100000., 0.), (draught_min, 0.)];
        if let Some(full_mesh) = self.mesh.as_ref() {
            //    let (vertices, indices) = (full_mesh.vertices().to_vec(), full_mesh.indices().to_vec());
            //    let full_mesh = TriMesh::with_flags(vertices, indices, TriMeshFlags::all()).unwrap();
            let full_volume = utils::volume(&full_mesh);
            let mut current_step = step / 30.; // сначала идем с маленьким шагом
            // на маленьких осадках кривая не линейная
            let mut draught = draught_min + current_step;
            while draught < draught_max {
                let result = full_mesh.local_split(&Vector::z_axis(), draught, self.epsilon);
                let volume = match result {
                    parry3d_f64::query::SplitResult::Pair(mut mesh, _) => {
                        if let Err(error) = mesh
                            .set_flags(TriMeshFlags::all())
                            .map_err(|err| error.pass_with("mesh.set_flags", err.to_string()))
                        {
                            log::error!("{}", error);
                        }
                        //    let filename = format!("10_{:.3}.stl", draught);
                        //    let cache_dir: PathBuf = ("src/algorithm/entities/model_cached/test/sofia/disp_bounded/195/stl/".to_owned() + &filename).into();
                        //    super::write_stl(&cache_dir, &mesh);
                        utils::volume(&mesh)
                    }
                    parry3d_f64::query::SplitResult::Negative => 0.,
                    parry3d_f64::query::SplitResult::Positive => full_volume,
                };
                steps.push((draught, volume));
                draught += if current_step < step {
                    current_step *= 1.5;
                    if current_step > step {
                        current_step = step;
                    }
                    current_step
                } else {
                    step
                };
            }
            //     dbg!(volume, full_volume);
            steps.push((draught_max, full_volume));
            steps.push((draught_max + 1000000., full_volume));
        } else {
            steps.push((draught_max + 1000000., 0.));
        }
        Ok(steps)
    }
    ///
    /// Расчет [длинны и ширины по ватерлинии](https://github.com/a-givertzman/sss/blob/6d91fb09de073995c3a165ebaaa76e4f1e202f36/design/algorithm/part04_stability/chapter05_criteria/section02_weatherCriteria.md)
    pub fn waterline_size(&self, draught: f64) -> Result<(f64, f64), Error> {
        let error = Error::new(&self.dbg, "waterline_size");
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
                let (mut vx, mut vy): (Vec<_>, Vec<_>) =
                    vertices.iter().map(|p| (p.x, p.y)).unzip();
                vx.sort_by(|&a, &b| a.partial_cmp(&b).unwrap());
                vy.sort_by(|&a, &b| a.partial_cmp(&b).unwrap());
                let min_x = vx.first().unwrap_or(&0.);
                let max_x = vx.last().unwrap_or(&0.);
                let min_y = vy.first().unwrap_or(&0.);
                let max_y = vy.last().unwrap_or(&0.);
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
        //     println!("{} {heel} {trim} {draught} start inertia", self.dbg);
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
                let (mut min_x, mut max_x, mut min_y, mut max_y) =
                    (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
                let vertices: Vec<_> = polyline
                    .vertices()
                    .iter()
                    .map(|p| position.transform_point(&p))
                    .map(|p| {
                        min_x = min_x.min(p.x);
                        max_x = max_x.max(p.x);
                        min_y = min_y.min(p.y);
                        max_y = max_y.max(p.y);
                        Point2::new(p.x, p.y)
                    })
                    .collect();
                let indices = polyline.indices();
                //        dbg!(&vertices, &indices);
                let max_delta = (max_y - min_y).max(max_x - min_x) as u32;
                let resolution = (max_delta * 100).min(self.resolution);
                if resolution < 2 {
                    return Ok((0., 0.));
                }
                /*        dbg!(
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    max_delta,
                    resolution,
                    self.resolution
                );*/
                let mut voxel_set = parry2d_f64::transformation::voxelization::VoxelSet::voxelize(
                    &vertices,
                    indices,
                    resolution,
                    parry2d_f64::transformation::voxelization::FillMode::FloodFill {
                        detect_cavities: false,
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
                //      dbg!(scale, voxels_volume, voxel_volume, voxel_area_center_x, voxel_area_center_y, max_bb);
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
    ///
    pub fn save(&self, path: &PathBuf) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "save");
        let mesh = self.mesh.as_ref().ok_or(error.err("no mesh"))?;
        super::write_stl(path, &mesh).map_err(|err| error.pass_with("write_stl", err))
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
