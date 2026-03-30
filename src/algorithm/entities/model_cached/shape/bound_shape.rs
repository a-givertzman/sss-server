use parry3d_f64::shape::TriMesh;
use sal_core::{dbg::Dbg, error::Error};

use crate::{algorithm::entities::model_cached::triangle_submerged_volume, kernel::types::Arc};

pub struct BoundShape {
    dbg: Dbg,
    parent_mesh: Arc<Option<TriMesh>>, // Используем Arc для разделения владения мешем
    indices: Vec<u32>,                 // Индексы треугольников, попавших в баунд
    position_x: f64,
    epsilon: f64,
}
//
impl BoundShape {
    //
    pub fn new(
        parent: Dbg,
        parent_mesh: &Arc<Option<TriMesh>>,
        indices: Vec<u32>,
        position_x: f64,
        epsilon: f64,
    ) -> Self {
        let dbg = Dbg::new(parent, "BoundShape");
        Self {
            dbg,
            parent_mesh: Arc::clone(parent_mesh),
            indices,
            position_x,
            epsilon,
        }
    }
    //
    pub fn displacement_by_steps(&self, step: f64) -> Result<Vec<(f64, f64)>, Error> {
        let error = Error::new(&self.dbg, "displacement_by_steps");
        let mesh = self
            .parent_mesh
            .as_ref()
            .clone()
            .ok_or(error.err("no mesh"))?;
        // Находим реальные границы Z для этой группы треугольников
        let (mut z_min, mut z_max) = (f64::MAX, f64::MIN);
        for &idx in &self.indices {
            let tri = mesh.triangle(idx);
            z_min = z_min.min(tri.a.z).min(tri.b.z).min(tri.c.z);
            z_max = z_max.max(tri.a.z).max(tri.b.z).max(tri.c.z);
        }

        let mut steps = vec![(-100000., 0.), (z_min, 0.)];
        let full_volume = self.indices
            .iter()
            .map(|&idx| {
                let tri = mesh.triangle(idx);
                // Знаковый объем тетраэдра (v1 x v2) · v3 / 6
                tri.a.coords.cross(&tri.b.coords).dot(&tri.c.coords) / 6.0
            })
            .sum::<f64>()
            .abs();

        let mut current_step = step / 30.;
        let mut draught = z_min + current_step;

        while draught < z_max {
            // Аналитическое суммирование без нарезки меша
            let volume: f64 = self
                .indices
                .iter()
                .map(|&idx| {
                    let tri = mesh.triangle(idx);
                    triangle_submerged_volume(&tri, draught)
                })
                .sum::<f64>()
                .abs();

            steps.push((draught, volume));

            // Сохраняем вашу логику адаптивного шага
            if current_step < step {
                current_step = (current_step * 1.5).min(step);
            }
            draught += current_step;
        }

        steps.push((z_max, full_volume));
        steps.push((z_max + 1000000., full_volume));

        Ok(steps)
    }
}
