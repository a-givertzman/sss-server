use crate::algorithm::entities::model_cached::{Shape, compartment_center, load_stl, write_stl};
use nalgebra::*;
use parry3d_f64::shape::{TriMesh, TriMeshFlags};
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use std::path::PathBuf;
///
/// Примитив для расчета площади горизонтальной поверхности фигуры
#[derive(Clone)]
pub struct AreaShape {
    dbg: Dbg,
    mesh: Option<TriMesh>,
    path: Option<PathBuf>,
    additional_path: Option<PathBuf>,
    center: Option<Point3<f64>>,
    scale: f64,
    resolution: u32,
    voxels: Option<Vec<(f64, Vec<f64>)>>,
    voxel_scale: Option<f64>,
}

unsafe impl Send for AreaShape {}

impl AreaShape {
    /// Конструктор
    /// * parent - Dbg родителя
    /// * mesh - модель
    /// * path - путь к файлу, содержащему модель
    /// * additional_path - путь к директории, содержащей дополнительные модели
    /// * model_x - смещение миделя относительно центра координат модели
    /// * scale - масштаб модели для ее приведения к метрам (1000: модель в мм)
    /// * resolution - точность расчета площади парусности
    /// * voxels - силуэт разбитый на квадратные примитивы - воксели,
    /// [смещение по х относительно center, [массив координат вокселей по z]]
    /// * voxel_scale - размер вокселя
    pub fn new(
        parent: &Dbg,
        mesh: Option<TriMesh>,
        path: Option<PathBuf>,
        additional_path: Option<PathBuf>,
        model_x: Option<f64>,
        scale: f64,
        resolution: u32,
        voxels: Option<Vec<(f64, Vec<f64>)>>,
        voxel_scale: Option<f64>,
    ) -> Self {
        let dbg = Dbg::new(parent, "Shape");
        Self {
            dbg,
            mesh,
            path,
            additional_path,
            center: model_x.map(|x| Point3::new(x, 0., 0.)),
            scale,
            resolution,
            voxels,
            voxel_scale,
        }
    }
    /// Конструктор для создания "ленивого" экземпляра.
    /// После создания обязателен вызов метода "init".
    /// center - смещение центра координат для расчетов относительно центра координат меша,
    /// для отсеков задается как None и считается автоматом
    pub fn new_uninit(
        parent: &Dbg,
        path: PathBuf,
        additional_path: Option<PathBuf>,
        model_x: Option<f64>,
        scale: f64,
    ) -> Self {
        Self::new(
            parent,
            None,
            Some(path),
            additional_path,
            model_x,
            scale,
            2000,
            None,
            None,
        )
    }
    /// Разбиваем поверхность меша на воксели и строим силуэт
    pub fn _voxelize(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "voxelize");
        let mesh = self.mesh.as_ref().ok_or(error.err("no mesh"))?;
        (self.voxels, self.voxel_scale) = {
            let aabb = mesh.local_aabb();
            let (dx, dz) = {
                let center = self.center.as_ref().unwrap();
                (aabb.mins.coords.x - center.x, aabb.mins.coords.z - center.z)
            };
            // разбиваем поверхность полученного над водой объема на воксели
            let voxel_set = parry3d_f64::transformation::voxelization::VoxelSet::voxelize(
                &mesh.vertices(),
                &mesh.indices(),
                self.resolution,
                parry3d_f64::transformation::voxelization::FillMode::SurfaceOnly,
                false,
            );
            let mut voxels = voxel_set.voxels().to_vec();
            // сортируем воксели по х
            voxels.sort_by(|a, b| a.coords.x.cmp(&b.coords.x));
            let mut current_max_x = 0;
            let mut result = Vec::new();
            let mut current = Vec::new();
            let scale = voxel_set.scale;
            let x = |x: u32| x as f64 * scale + dx;
            let z = |z: u32| z as f64 * scale + dz;
            // проходим по вокселям по порядку и берем воксели с одинаковой координатой по x,
            // отбрасываем с одинаковой координатой по y, полчаем боковую поверхность
            for p in voxels.iter() {
                if p.coords.x > current_max_x {
                    current.sort();
                    current.dedup();
                    result.push((
                        x(current_max_x),
                        current.iter().map(|v: &u32| z(*v)).collect(),
                    ));
                    current = Vec::new();
                    current_max_x += 1;
                    while p.coords.x > current_max_x {
                        result.push((x(current_max_x), Vec::new()));
                        current_max_x += 1;
                    }
                }
                current.push(p.coords.z);
            }
            current.sort();
            current.dedup();
            result.push((x(current_max_x), current.iter().map(|&v| z(v)).collect()));
            (Some(result), Some(scale))
        };
        Ok(())
    }
    /// Расчет поверхности парусности
    /// Возвращает повернутое и смещенное разбиение [dx, area]
    pub fn windage_area_data(&self, draught: f64) -> Result<Vec<(f64, Vec<(f64, f64)>)>, Error> {
        let error = Error::new(&self.dbg, "windage_area_data");
        let voxels = self.voxels.as_ref().ok_or(error.err("no voxels"))?;
        let voxel_scale = self.voxel_scale.ok_or(error.err("no voxel_scale"))?;
        let voxel_area = voxel_scale * voxel_scale;
        let center = self.center.ok_or(error.err("no center"))?;
        let result: Vec<_> = voxels
            .iter()
            .map(|(x, v)| {
                (
                    *x + center.x,
                    v.iter()
                        .map(|z| (z + center.z - draught, voxel_area))
                        .filter(|(z, _)| *z >= 0.)
                        .collect(),
                )
            })
            .collect();
        Ok(result)
    }
}
//
impl Shape for AreaShape {
    /// Init shape, load geometry
    fn init(&mut self) -> Result<(), Error> {
        if self.mesh.is_none() {
            let error = Error::new(&self.dbg, "init");
            let mut mesh = if let Some(additional_path) = self.additional_path.clone() {
                let path_fixed = additional_path.join("hull_fixed.stl");
                if let Ok(mesh) = load_stl(&path_fixed) {
                    mesh
                } else {
                    let mut mesh = load_stl(&self.path.clone().ok_or(error.err("empty path"))?)
                        .map_err(|err| error.pass_with("load", err.to_string()))?;
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
                        .map(|p| load_stl(&p))
                        .partition(|r: &Result<_, Error>| r.is_ok());
                    meshes.into_iter().for_each(|m| mesh.append(&m.unwrap()));
                    let (indices, vertices) = (mesh.indices().to_vec(), mesh.vertices().to_vec());
                    let mesh = TriMesh::with_flags(vertices, indices, TriMeshFlags::all())
                        .map_err(|err| error.pass_with("TriMesh::with_flags", err.to_string()))?;
                    write_stl(&path_fixed, &mesh)
                        .map_err(|err| error.pass_with("write_stl", err.to_string()))?;
                    mesh
                }
            } else {
                load_stl(&self.path.clone().ok_or(error.err("empty path"))?)
                    .map_err(|err| error.pass_with("load", err.to_string()))?
            };
            let scale = 1. / self.scale;
            mesh = mesh.scaled(&Vector3::new(scale, scale, scale));
            if self.center.is_none() {
                self.center = Some(compartment_center(&mesh));
            }
            self.mesh = Some(mesh);
            self._voxelize()
                .map_err(|err| error.pass_with("self.voxelize", err))?;
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
