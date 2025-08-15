use nalgebra::*;
use obj::{Obj, ObjData};
use parry3d_f64::bounding_volume::Aabb;
use parry3d_f64::shape::{Cuboid, TriMesh, TriMeshFlags};
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::algorithm::entities::{Bounds, Position};

#[derive(Clone)]
pub struct Shape {
    dbg: Dbg,
    mesh: Option<TriMesh>,
    path: Option<PathBuf>,
    additional_path: Option<PathBuf>,
    delta_pos: Option<Point3<f64>>,
    scale: f64,
    epsilon: f64,
    resolution_inertia: u32,
    resolution_windage_area: u32,
}

unsafe impl Send for Shape {}

impl Shape {
    /// Конструктор
    /// * parent - Dbg родителя
    /// * mesh - модель
    /// * path - путь к файлу, содержащему модель
    /// * additional_path - путь к директории, содержащей дополнительные модели
    /// * dx - смещение миделя относительно центра координат модели
    /// * scale - масштаб модели для ее приведения к метрам (1000: модель в мм)
    /// * epsilon - точность расчета сечений
    /// * resolution_inertia - точность расчета момента инерции
    /// * resolution_vertical_area - точность расчета площади парусности
    pub fn new(
        parent: &Dbg,
        mesh: Option<TriMesh>,
        path: Option<PathBuf>,
        additional_path: Option<PathBuf>,
        delta_pos: Option<Point3<f64>>,
        scale: f64,
        epsilon: f64,
        resolution_inertia: u32,
        resolution_vertical_area: u32,
    ) -> Self {
        let dbg = Dbg::new(parent, "Shape");
        Self {
            dbg,
            mesh,
            path,
            additional_path,
            delta_pos,
            scale,
            epsilon,
            resolution_inertia,
            resolution_windage_area: resolution_vertical_area,
        }
    }
    /// Конструктор для создания "ленивого" экземпляра.
    /// После создания обязателен вызов метода "init".
    /// delta_pos - смещение центра координат, для отсеков задается как None и считается автоматом
    pub fn new_uninit(
        parent: &Dbg,
        path: PathBuf,
        additional_path: Option<PathBuf>,
        delta_pos: Option<Position>,
        scale: f64,
    ) -> Self {
        Self::new(
            parent,
            None,
            Some(path),
            additional_path,
            delta_pos.map(|p| Point3::new(p.x(), p.y(), p.z())),
            scale,
            0.0000001,
            10000,
            2000,
        )
    }
    /// Init shape, load geometry
    pub fn init(&mut self) -> Result<(), Error> {
        if self.mesh.is_none() {
            let error = Error::new(&self.dbg, "init");
            let mut mesh = load_stl(self.path.clone().ok_or(error.err("empty path"))?)
                .map_err(|err| error.pass_with("load", err.to_string()))?;
            if let Some(additional_path) = self.additional_path.clone() {
                let dir = std::fs::read_dir(additional_path)
                    .map_err(|err| error.pass_with("read additional dir", err.to_string()))?;
                let pathes: Vec<_> = dir
                    .into_iter()
                    .filter_map(|f| f.ok())
                    .map(|f| f.path())
                    .collect();
                let (
                    meshes,
                    _errors, // TODO: подумать, что делать с этими ошибками
                ): (Vec<_>, Vec<_>) = pathes
                    .into_iter()
                    .map(|p| load_stl(p))
                    .partition(|r| r.is_ok());
                meshes.into_iter().for_each(|m| mesh.append(&m.unwrap()));
            }
            let scale = 1. / self.scale;
            mesh = mesh.scaled(&Vector3::new(scale, scale, scale));
            if self.delta_pos.is_none() {
                self.delta_pos = Some(compartment_center(&mesh));
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
        let position = self.position(heel, trim, draught);
        let cuboid_half_size = 1000.;
        let cuboid = Cuboid::new(Vector3::repeat(cuboid_half_size));
        let result = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .intersection_with_cuboid(
                &position.map_err(|err| error.pass_with("position", err))?,
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
                    return Err(error.err("mesh.intersection_with_plane error: no intersection!"));
                }
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
        let position = self.position(heel, trim, draught);
        let cuboid_half_size = 1000.;
        let hdz = 0.005;
        let cuboid = Cuboid::new(Vector3::new(cuboid_half_size, cuboid_half_size, hdz));
        let result = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .intersection_with_cuboid(
                &position.map_err(|err| error.pass_with("position", err))?,
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
                    return Err(error.err("mesh.intersection_with_plane error: no intersection!"));
                }
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
        match result {
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
                return Ok((dx, dy));
            }
            _ => return Err(error.err("mesh.intersection_with_plane error!")),
        };
    }
    ///
    /// Расчет [момента инерции свободной поверхности жидкости](https://github.com/a-givertzman/sss/blob/cdef1e9a2133adeb2fe8abcda6229b206c28493c/design/algorithm/part04_stability/chapter01_initialStability/chapter01_initialStability.md#%D0%B2%D0%BB%D0%B8%D1%8F%D0%BD%D0%B8%D0%B5-%D1%81%D0%B2%D0%BE%D0%B1%D0%BE%D0%B4%D0%BD%D0%BE%D0%B9-%D0%BF%D0%BE%D0%B2%D0%B5%D1%80%D1%85%D0%BD%D0%BE%D1%81%D1%82%D0%B8)
    pub fn inertia(&self, heel: f64, trim: f64, draught: f64) -> Result<(f64, f64), Error> {
        let error = Error::new(&self.dbg, "inertia");
        let position = self
            .position(heel, trim, draught)
            .map_err(|err| error.pass_with("position", err))?;
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
                    self.resolution_inertia,
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
                return Ok((i_x, i_y));
            }
            _ => return Err(error.err("mesh.intersection_with_plane error!")),
        };
    }
    /// Расчет поверхности парусности
    /// Возвращает [шкала, баундбокс, разбиение с количеством вокселей по х]
    fn _windage_area(&self, trim: f64, draught: f64) -> Result<(f64, Aabb, Vec<usize>), Error> {
        let error = Error::new(&self.dbg, "_windage_area");
        let position = self
            .position(0., trim, draught)
            .map_err(|err| error.pass_with("position", err))?;
        let cuboid_half_size = 1000.;
        let cuboid = Cuboid::new(Vector3::repeat(cuboid_half_size));
        // берем часть корпса над водой как перечечение модели и кубика, имитирующего воду
        let result = self
            .mesh
            .as_ref()
            .ok_or(error.err("no mesh"))?
            .intersection_with_cuboid(
                &position,
                false,
                &cuboid,
                &Isometry::from_parts(
                    Translation3::new(0., 0., cuboid_half_size),
                    UnitQuaternion::identity(),
                ),
                false,
                self.epsilon,
            );
        let mesh = match result {
            Ok(mesh) => match mesh {
                Some(mesh) => mesh,
                None => {
                    return Err(error.err("mesh.intersection_with_plane error: no intersection!"));
                }
            },
            Err(e) => return Err(error.pass_with("mesh.intersection_with_plane", e.to_string())),
        };
        let aabb = mesh.local_aabb();
        // разбиваем поверхность полученного над водой объема на воксели
        let voxel_set = parry3d_f64::transformation::voxelization::VoxelSet::voxelize(
            &mesh.vertices(),
            &mesh.indices(),
            self.resolution_windage_area,
            parry3d_f64::transformation::voxelization::FillMode::SurfaceOnly,
            false,
        );
        let mut voxels = voxel_set.voxels().to_vec();
        // сортируем воксели по х
        voxels.sort_by(|a, b| a.coords.x.cmp(&b.coords.x));
        let mut current_max_x = 0;
        let mut result_z = Vec::new();
        let mut current_z = Vec::new();
        // проходим по вокселям по порядку и берем воксели с одинаковой координатой по x,
        // отбрасываем с одинаковой координатой по y, полчаем боковую поверхность
        for p in voxels.iter() {
            if p.coords.x > current_max_x {
                current_z.sort();
                current_z.dedup();
                result_z.push(current_z.len());
                current_z = Vec::new();
                current_max_x += 1;
                while p.coords.x > current_max_x {
                    current_max_x += 1;
                    result_z.push(0);
                }
            }
            current_z.push(p.coords.z);
        }
        Ok((voxel_set.scale, aabb, result_z))
    }
    /// Расчет площади и центра площади парусности
    /// Возвращает [площадь, смещение площади по x]
    pub fn windage_area(&self, trim: f64, draught: f64) -> Result<(f64, f64), Error> {
        let error = Error::new(&self.dbg, "windage_area");
        let (scale, aabb, result_z) = self
            ._windage_area(trim, draught)
            .map_err(|e| error.pass_with("_windage_area", e.to_string()))?;
        let mut area = 0;
        let mut moment = 0;
        for (x, &dz) in result_z.iter().enumerate() {
            moment += x * dz;
            area += dz;
        }
        let center_x = (moment as f64 / area as f64 * scale) + aabb.mins.x;
        let area = area as f64 * scale * scale;
        Ok((area, center_x))
    }
    // TODO: можно как-то объеденить с расчетом поверхности, но возникают сложности с кэшами
    /// Расчет распределения площади парусности
    /// Возвращает набор значений (начало площади по x, конец площади по x, массив значений площади)
    pub fn bounded_windage_area(
        &self,
        trim: f64,
        draught: f64,
    ) -> Result<(f64, f64, Vec<f64>), Error> {
        let error = Error::new(&self.dbg, "bounded_windage_area");
        let base_aabb = self.mesh.clone().ok_or(error.err("no mesh"))?.local_aabb();
        // набор значений площади в разбиении по площади части модели над водой
        let (scale, result_aabb, result_z) = self
            ._windage_area(trim, draught)
            .map_err(|e| error.pass_with("_windage_area", e.to_string()))?;
        let base_bounds = Bounds::from_min_max(
            base_aabb.mins.coords.x,
            base_aabb.maxs.coords.x,
            self.resolution_windage_area as usize,
        )
        .map_err(|e| error.pass_with("Bounds::from_min_max", e))?;
        let area_bounds = Bounds::from_min_max(
            result_aabb.mins.coords.x,
            result_aabb.maxs.coords.x,
            result_z.len(),
        )
        .map_err(|e| error.pass_with("Bounds::from_min_max", e))?;
        let mut area_bounds_map = HashMap::new();
        for (index, (area_bound, area)) in area_bounds.iter().zip(result_z.iter()).enumerate() {
            area_bounds_map.insert(index, (area_bound, area));
        }
        let area_scale = scale * scale;
        let mut base_area_result = Vec::new();
        // пересчет разбиения относительно исходной модели
        // проходим по разбиению исходной модели и проверяем попадание частей разбиения по модели над поверхностью воды
        for base_bound in base_bounds.iter() {
            let mut base_area = 0.;
            let mut last_index = 0;
            for index in last_index..result_z.len() {
                let (area_bound, area) = area_bounds_map
                    .get(&index)
                    .ok_or(error.err("area_bounds_map.get(index)"))?;
                if area_bound.start() >= base_bound.end() {
                    last_index = index;
                    break;
                }
                base_area += **area as f64
                    * base_bound
                        .part_ratio(area_bound)
                        .map_err(|e| error.pass_with("base_bound.part_ratio", e))?;
            }
            base_area_result.push(base_area * area_scale);
        }
        Ok((
            base_aabb.mins.coords.x,
            base_aabb.maxs.coords.x,
            base_area_result,
        ))
    }
    ///
    /// Расчет положения корпуса
    fn position(&self, heel: f64, trim: f64, draught: f64) -> Result<Isometry3<f64>, Error> {
        let error = Error::new(&self.dbg, "position");
        let heel_rad = -heel.to_radians();
        let trim_rad = trim.to_radians();
        let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
        let transformed_x_axis = trim_rotation.transform_point(&Point3::new(1., 0., 0.));
        let transformed_x_axis = UnitVector3::new_normalize(Vector3::new(
            transformed_x_axis.x,
            transformed_x_axis.y,
            transformed_x_axis.z,
        ));
        let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
        let rotation = heel_rotation * trim_rotation;
        let mut center = self.delta_pos.ok_or(error.err("no delta_pos"))?;
        center.z += draught;
        let point = rotation.transform_point(&center);
        let translation = Translation3::new(-point.x, -point.y, -point.z);
        Ok(Isometry::from_parts(translation, rotation))
    }
    /// Разбиение от draught_min до h_max меша на draught_step шаги
    pub fn draught_steps(
        &self,         
        draught_min: f64,
        draught_step: f64,
    ) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "height");
        if draught_step <= 0. {
            return Err(error.err("draught_step == 0"))
        }
        let aabb = self.mesh.clone().ok_or(error.err("no mesh"))?.local_aabb();
        if draught_min >= aabb.maxs.z {
            return Err(error.err("draught_min >= h_max"))
        }
        let mut result = vec![];
        let mut current = draught_min;
        while current < aabb.maxs.z {
            result.push(current);
            current += draught_step;
        } 
        result.push(aabb.maxs.z);
        Ok(result)
    }
}
///
/// Расчет начала координат для отсеков как
/// проекции центра объема модели на ее нижнюю плоскость
fn compartment_center(mesh: &TriMesh) -> Point3<f64> {
    let properties = parry3d_f64::shape::Shape::mass_properties(mesh, 1.);
    let aabb: Aabb = mesh.local_aabb();
    Point3::new(properties.local_com.x, properties.local_com.y, aabb.mins.z)
}
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
pub fn load_stl(path: PathBuf) -> Result<TriMesh, Error> {
    let error = Error::new("Shape", "load_obj");
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
