pub struct BoundShape {
    dbg: String,
    parent_mesh: Arc<TriMesh>, // Используем Arc для разделения владения мешем
    indices: Vec<u32>,         // Индексы треугольников, попавших в баунд
    position_x: f64,
    epsilon: f64,
    resolution: f64,
}
// 
impl BoundShape {  
    pub fn displacement_by_steps(&self, step: f64) -> Result<Vec<(f64, f64)>, Error> {
        let error = Error::new(&self.dbg, "displacement_by_steps");
        
        // Находим реальные границы Z для этой группы треугольников
        let (mut z_min, mut z_max) = (f64::MAX, f64::MIN);
        for &idx in &self.indices {
            let tri = self.parent_mesh.triangle(idx);
            z_min = z_min.min(tri.a.z).min(tri.b.z).min(tri.c.z);
            z_max = z_max.max(tri.a.z).max(tri.b.z).max(tri.c.z);
        }

        let mut steps = vec![(-100000., 0.), (z_min, 0.)];
        let full_volume = self.calculate_total_volume();

        let mut current_step = step / 30.;
        let mut draught = z_min + current_step;

        while draught < z_max {
            // Аналитическое суммирование без нарезки меша
            let volume: f64 = self.indices.iter()
                .map(|&idx| {
                    let tri = self.parent_mesh.triangle(idx);
                    triangle_submerged_volume(&tri, draught)
                })
                .sum::<f64>().abs();

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

    fn calculate_total_volume(&self) -> f64 {
        self.indices.iter()
            .map(|&idx| {
                let tri = self.parent_mesh.triangle(idx);
                // Знаковый объем тетраэдра (v1 x v2) · v3 / 6
                tri.a.coords.cross(&tri.b.coords).dot(&tri.c.coords) / 6.0
            })
            .sum::<f64>().abs()
    }
}