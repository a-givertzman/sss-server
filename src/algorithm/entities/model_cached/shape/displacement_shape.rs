use log::Log;
use nalgebra::*;
use parry3d_f64::shape::{Cuboid, Shape as _, TriMesh, TriMeshFlags};
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use crate::algorithm::entities::model_cached::shape::utils;
use crate::algorithm::entities::model_cached::{Shape, compartment_center, load_stl, volume, write_stl};
use crate::algorithm::entities::{Bound, Position};

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
    pub fn part(&self, bound: &Bound) -> Result<Option<Self>, Error> {
        let error = Error::new(&self.dbg, "split");
        let half_size_x = bound.length().ok_or(error.err("no bound.length"))? / 2.;
        //  let center_x = self.center.unwrap_or(Point3::new(0., 0., 0.)).x;
        //  let position_x = bound.center().ok_or(error.err("no bound.center"))? + center_x;
        let position_x = bound.center().ok_or(error.err("no bound.center"))?;
        let cuboid = Cuboid::new(Vector3::new(half_size_x, 100000., 100000.));
        let mut src_mesh = self.mesh.as_ref().ok_or(error.err("no mesh"))?;
        let mut mesh;
        let mut epsilon = self.epsilon;
        loop {
            // TODO костыль для фикса бага: иногда меш вырезается не целиком.
            // Тут проверяется что баунд вырезанного меша не превышает шаблон
            let result = src_mesh.intersection_with_local_cuboid(
                false,
                &cuboid,
                &Isometry::from_parts(
                    Translation3::new(position_x, 0., 0.),
                    UnitQuaternion::identity(),
                ),
                false,
                epsilon,
            );
            mesh = match result {
                Ok(mesh) => match mesh {
                    Some(mesh) => mesh,
                    None => return Ok(None),
                },
                Err(e) => {
                    return Err(
                        error.pass_with("mesh.intersection_with_local_cuboid", e.to_string())
                    );
                }
            };
            let aabb = mesh.aabb(&Isometry::identity());
            let bound_x_min = position_x - half_size_x;
            let bound_x_max = position_x + half_size_x;
            if aabb.mins.x + epsilon < bound_x_min || aabb.maxs.x - epsilon > bound_x_max {
                let error = format!(
                    "{} part error: wrong aabb, rebuild! x:{position_x} b_min:{bound_x_min} b_max:{bound_x_max} aabb.min:{} aabb.max:{} epsilon:{}",
                    self.dbg, aabb.mins.x, aabb.maxs.x, epsilon
                );
                log::warn!("{error}");
                src_mesh = &mesh;
                epsilon = epsilon * 10.;
                continue;
            }
            break;
        }
        if let Err(error) = mesh
            .set_flags(TriMeshFlags::all())
            .map_err(|err| error.pass_with("mesh.set_flags", err.to_string()))
        {
            log::error!("{}", error);
        }
        //  let filename = format!("{:.1}, {:.1}.stl", bound.start().unwrap() + 65.25,  bound.end().unwrap() + 65.25,);
        //  let cache_dir: PathBuf = ("src/algorithm/entities/model_cached/test/sofia/disp_bounded/195/stl/".to_owned() + &filename).into();
        //  super::write_stl(&cache_dir, &mesh);
        Ok(Some(Self::new(
            &self.dbg,
            Some(mesh),
            None,
            Some(position_x),
            1.,
            epsilon,
            self.resolution,
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
        let position = self
            .position(heel, trim, draught)
            .map_err(|err| error.pass_with("self.position", err))?;
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
    /// Расчёт площади и центра ватерлинии на заданной осадке.
    ///
    /// Метод выполняет геометрическое сечение 3D-мешa горизонтальной плоскостью
    /// на уровне `draught` и вычисляет характеристики полученной ватерлинии.
    /// - `mesh` — треугольная поверхность корпуса (`TriMesh`);
    /// - `draught` — осадка, на которой выполняется сечение [м].
    pub fn waterline(
        &self,
        mesh: &TriMesh,
        draught: f64
    ) -> (f64, Position) {
        let cuboid_half_size = 1000.;
        let hdz = 0.005;
        let cuboid = Cuboid::new(Vector3::new(cuboid_half_size, cuboid_half_size, hdz));
        let plane_pos = Isometry::<f64, nalgebra::UnitQuaternion<f64>, 3>::translation(0.0, 0.0, draught);
        let wl_result = mesh.intersection_with_cuboid(
            &Isometry::identity(),
            false,
            &cuboid,
            &plane_pos,
            false,
            self.epsilon,
        );
        let (wl_area, wl_center) = match wl_result {
            Ok(Some(mut wl_mesh)) => {
                let _ = wl_mesh.set_flags(TriMeshFlags::all());
                let (area, pos) = super::properties(&wl_mesh, 0.5 / hdz);
                (area, pos)
            },
            _ => (0.0, Position::new(0.0, 0.0, draught)),
        };
        (wl_area, wl_center)
    }
    ///
    /// Подсчёт момента инерции
    fn calculate_inertia(&self, wl_mesh: &TriMesh, draught: f64) -> Result<(f64, f64), Error> {
        let result = wl_mesh
            .intersection_with_plane(&Isometry::identity(), &Vector3::z_axis(), draught, self.epsilon);
        match result {
            parry3d_f64::query::IntersectResult::Intersect(polyline) => {
                let (mut min_x, mut max_x, mut min_y, mut max_y) =
                    (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
                let vertices: Vec<_> = polyline
                    .vertices()
                    .iter()
                    .map(|p| {
                        min_x = min_x.min(p.x);
                        max_x = max_x.max(p.x);
                        min_y = min_y.min(p.y);
                        max_y = max_y.max(p.y);
                        Point2::new(p.x, p.y)
                    })
                    .collect();
                let indices = polyline.indices();
                let max_delta = (max_y - min_y).max(max_x - min_x) as u32;
                let resolution = (max_delta * 100).min(self.resolution);
                if resolution < 2 {
                    return Ok((0., 0.));
                }
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
    /// Вычисляет гидростатические параметры точки начала отсчета (KDP) и формирует шаги осадки.
    /// * `lowest_center_point` — Центр величины или геометрический центр (локальный).
    /// * `mp_center_point` — Проекция центра на основную плоскость (Baseline, Z=0).
    /// * `high_mesh_point` — Самая высокая точка меша (локальный максимум Z).
    /// * `low_mesh_point` — Самая низкая точка меша в мировых координатах после наклона.
    /// result: `(baseline_distance, start_point_world)`
    pub fn main_surface_distance(
        &self,
        ship_to_world: &Isometry<f64, Unit<Quaternion<f64>>, 3>,
        lowest_center_point: OPoint<f64, Const<3>>,
        mp_center_point: OPoint<f64, Const<3>>,
        low_mesh_point_z: f64,
    ) -> (f64, OPoint<f64, Const<3>>) {
        let center_world = ship_to_world.transform_point(&lowest_center_point);
        let baseline_proj_world = ship_to_world.transform_point(&mp_center_point);
        let axis_direction = baseline_proj_world - center_world;
        // t = (Z_цели - Z_старта) / V_z
        let t = (low_mesh_point_z - center_world.z) / axis_direction.z;
        let intersection_world = center_world + axis_direction * t;
        // Расстояние в мировых координатах (между точкой касания и проекцией ОП)
        let baseline_distance = nalgebra::distance(&intersection_world, &baseline_proj_world);
        (baseline_distance, intersection_world)
    }
    ///
    /// Вычисляет дискретные точки изменения ватерлинии при погружении/осадке судна.
    ///
    /// Функция строит последовательность точек вдоль оси погружения (в мировых координатах),
    /// начиная от заданной начальной ватерлинии и заканчивая заданной отметкой по оси Z.
    /// - `ship_to_world` — преобразование из локальной системы судна в мировую;
    /// - `start_waterline_point_world` — начальная точка ватерлинии в мировых координатах;
    /// - `lowest_center_point_local` — нижняя опорная точка в локальных координатах;
    /// - `mp_center_point_local` — опорная точка (например, МП) в локальных координатах;
    /// - `end_z_world` — конечная отметка по оси Z (мировая система координат);
    /// - `draught_step` — шаг дискретизации изменения осадки
    pub fn waterline_steps(
        &self,
        ship_to_world: &Isometry<f64, Unit<Quaternion<f64>>, 3>,
        start_waterline_point_world: OPoint<f64, Const<3>>, 
        lowest_center_point_local: OPoint<f64, Const<3>>,
        mp_center_point_local: OPoint<f64, Const<3>>,
        end_z_world: f64,
        draught_step: f64,
    ) -> Vec<OPoint<f64, Const<3>>> {
        let mut result = Vec::new();
        // Направление оси вдоль которой движемся (в мировых координатах)
        let center_world = ship_to_world.transform_point(&lowest_center_point_local);
        let baseline_proj_world = ship_to_world.transform_point(&mp_center_point_local);
        // Вектор оси от центра к ОП (нормированный)
        let axis_vec = (center_world - baseline_proj_world).normalize();
        let start = start_waterline_point_world;
        let mut curr_p = start;
        let mut next_dist_p = draught_step;
        loop {
            if curr_p.z >= end_z_world {
                break;
            }
            result.push(curr_p);
            curr_p = start + (axis_vec * next_dist_p);
            next_dist_p += draught_step;
        }
        // Добавляем финальную точку, соответствующую полному погружению
        // Чтобы найти её точно на оси, нужно решить: start_z + (axis_vec.z * t) = end_z
        let t_final = (end_z_world - start_waterline_point_world.z) / axis_vec.z;
        result.push(start_waterline_point_world + axis_vec * t_final);
        result
    }
    ///
    /// Пошаговый расчёт водоизмещения и характеристик плавучести.
    ///
    /// Выполняет дискретный анализ погружения 3D-модели корпуса судна
    /// при заданных углах крена и дифферента.
    /// - `heel` — угол крена [рад или град, в зависимости от модели];
    /// - `trim` — угол дифферента;
    /// - `draught_step` — шаг дискретизации по осадке.
    pub fn step_displacement(
        &self,
        heel: f64,
        trim: f64,
        draught_step: f64
    ) -> Result<Vec<(f64, OPoint<f64, Const<3>>, f64, OPoint<f64, Const<3>>, f64, f64, f64, f64)>, Error> {
        let error = Error::new(&self.dbg, "step_displacement");
        let mut mesh_in_world = self.mesh.as_ref().ok_or(error.err("no mesh"))?.clone();
        let (_, local_vol_center) = super::properties(&mesh_in_world, 1.0);
        let center_point_local = OPoint::<f64, Const<3>>::new(local_vol_center.x(), local_vol_center.y(), local_vol_center.z());
        let baseline_proj_local = OPoint::<f64, Const<3>>::new(center_point_local.x, center_point_local.y, 0.0);
        mesh_in_world.transform_vertices(&self.position_yz(heel, trim, 0.0).map_err(|err| error.pass_with("ship_to_world_transform", err))?);
        let ship_to_world = self.position_yz(-heel, trim, 0.0)
            .map_err(|err| error.pass_with("ship_to_world_transform", err))?;
        let world_to_ship = ship_to_world.inverse();
        let world_bounds = mesh_in_world.compute_local_aabb();
        let (dist_from_baseline, start_waterline_point) = self.main_surface_distance(
            &ship_to_world,
            center_point_local,
            baseline_proj_local,
            world_bounds.mins.z
        );
        let waterline_steps = self.waterline_steps(
            &ship_to_world, 
            start_waterline_point, 
            center_point_local, 
            baseline_proj_local, 
            world_bounds.maxs.z, 
            draught_step
        );
        let mut results = Vec::new();
        let cut_axis = Vector3::z_axis();
        for step in waterline_steps {
            let (wl_area, wl_center_world) = self.waterline(&mesh_in_world, step.z);
            let (ix, iy) = self.calculate_inertia(&mesh_in_world, step.z).unwrap_or((0.0, 0.0));
            let relative_draught = nalgebra::distance(&step, &start_waterline_point);
            match mesh_in_world.split(&Isometry::identity(), &cut_axis, step.z, self.epsilon) {
                parry3d_f64::query::SplitResult::Pair(submerged_part, _) => {
                    let (vol, vol_center_w) = super::properties(&submerged_part, 1.0);
                    let vol_center_ship = world_to_ship.transform_point(&OPoint::<f64, Const<3>>::new(vol_center_w.x(), vol_center_w.y(), vol_center_w.z()));
                    let wl_center_ship = world_to_ship.transform_point(&OPoint::<f64, Const<3>>::new(wl_center_world.x(), wl_center_world.y(), wl_center_world.z()));
                    results.push((
                        vol,
                        vol_center_ship,
                        wl_area,
                        wl_center_ship,
                        ix, iy,
                        relative_draught,
                        dist_from_baseline
                    ));
                },
                parry3d_f64::query::SplitResult::Negative => {
                    // Весь меш ниже уровня воды
                    let (vol, vol_center_w) = super::properties(&mesh_in_world, 1.0);
                    let vol_center_ship = world_to_ship.transform_point(&OPoint::<f64, Const<3>>::new(vol_center_w.x(), vol_center_w.y(), vol_center_w.z()));
                    results.push((
                        vol, 
                        vol_center_ship, 
                        wl_area, 
                        OPoint::<f64, Const<3>>::new(0.0, 0.0, 0.0), 
                        0.0, 
                        0.0, 
                        relative_draught, 
                        dist_from_baseline
                    ));
                },
                parry3d_f64::query::SplitResult::Positive => {
                    // Весь меш выше уровня воды
                    results.push((
                        0.0, 
                        OPoint::<f64, Const<3>>::new(0.0, 0.0, 0.0), 
                        wl_area, 
                        OPoint::<f64, Const<3>>::new(0.0, 0.0, 0.0), 
                        0.0, 
                        0.0, 
                        0.0, 
                        dist_from_baseline
                    ));
                },
            }
        }
        Ok(results)
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
    /// Полный размер модели (длина, ширина, высота, минимальная высота)
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
