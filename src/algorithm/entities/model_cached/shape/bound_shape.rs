use parry3d_f64::shape::{TriMesh, Triangle};
use sal_core::{dbg::Dbg, error::Error};

use crate::{algorithm::entities::model_cached::{generate_cap_triangles, triangle_submerged_volume}, kernel::types::Arc};

pub struct BoundShape {
    dbg: Dbg,
    // Все треугольники, составляющие ЗАМКНУТЫЙ объем (включая торцы)
    triangles: Vec<Triangle>,
    // Предрассчитанные полные объемы для каждого треугольника
    full_volumes: Vec<f64>,
    z_min: f64,
    z_max: f64,
}

impl BoundShape {
    pub fn new(
        parent: &Dbg,
        parent_mesh: &Arc<Option<TriMesh>>,
        indices: Vec<u32>,
        is_flipped: Vec<bool>,
        position_x: f64, // Центр баунда
        half_size_x: f64, // Половина длины (из метода part)
    ) -> Self {
        let mesh = parent_mesh.as_ref().as_ref().unwrap();
        let mesh_indices = mesh.indices();
        let vertices = mesh.vertices();
        
        let mut triangles = Vec::with_capacity(indices.len() + 32);
        let x_min = position_x - half_size_x;
        let x_max = position_x + half_size_x;

        // 1. Добавляем основные (боковые) треугольники
        for (&idx, &flip) in indices.iter().zip(is_flipped.iter()) {
            let tri = mesh.triangle(idx);
            triangles.push(if flip { Triangle::new(tri.a, tri.c, tri.b) } else { tri });
        }

        // 2. Генерируем "крышку" для левого торца (нормаль в -X)
        let left_cap = generate_cap_triangles(mesh_indices, vertices, &indices, x_min, false);
        triangles.extend(left_cap);

        // 3. Генерируем "крышку" для правого торца (нормаль в +X)
        let right_cap = generate_cap_triangles(mesh_indices, vertices, &indices, x_max, true);
        triangles.extend(right_cap);

        // 4. Предварительный расчет Z-границ и объемов (как в предыдущем ответе)
        let mut z_min = f64::MAX;
        let mut z_max = f64::MIN;
        let full_volumes: Vec<f64> = triangles.iter().map(|t| {
            z_min = z_min.min(t.a.z).min(t.b.z).min(t.c.z);
            z_max = z_max.max(t.a.z).max(t.b.z).max(t.c.z);
            t.a.coords.cross(&t.b.coords).dot(&t.c.coords) / 6.0
        }).collect();

        Self {
            dbg: Dbg::new(parent, "BoundShape"),
            triangles,
            full_volumes,
            z_min,
            z_max,
        }
    }

    pub fn displacement_by_steps(&self, step: f64) -> Result<Vec<(f64, f64)>, Error> {
        // Инициализация кривой
        let mut steps = vec![(-1e5, 0.0), (self.z_min, 0.0)];
        
        let mut current_step = (step / 30.0).max(0.001);
        let mut draught = self.z_min + current_step;

        // Кэшируем общее количество для итератора
        let tri_count = self.triangles.len();

        while draught < self.z_max {
            let mut total_volume = 0.0;

            for i in 0..tri_count {
                let tri = &self.triangles[i];
                
                // ОПТИМИЗАЦИЯ 1: Треугольник целиком над водой
                if tri.a.z > draught && tri.b.z > draught && tri.c.z > draught {
                    continue;
                }

                // ОПТИМИЗАЦИЯ 2: Треугольник целиком под водой (берем готовое число)
                if tri.a.z <= draught && tri.b.z <= draught && tri.c.z <= draught {
                    total_volume += self.full_volumes[i];
                    continue;
                }

                // ОПТИМИЗАЦИЯ 3: Только если пересекается, вызываем тяжелый клиппинг
                total_volume += triangle_submerged_volume(tri, draught);
            }

            steps.push((draught, total_volume.abs()));

            if current_step < step {
                current_step = (current_step * 1.5).min(step);
            }
            draught += current_step;
        }

        let final_vol: f64 = self.full_volumes.iter().sum::<f64>().abs();
        steps.push((self.z_max, final_vol));
        steps.push((1e6, final_vol));

        Ok(steps)
    }
}