use sal_core::{dbg::Dbg, error::Error};
use core::panic;
use std::fs;
use crate::algorithm::eval::entities::compartment_corner_points::CompartmentCornerPoints;
use crate::algorithm::eval::import_tanks::import_3d_tanks_ctx::Import3DTanksCtx;
use crate::{
    algorithm::{
        context::context_access::ContextReadRef, 
        eval::{
            Zg
        }
    }, 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{
        ContextWrite, 
        InitialCtx
    }
};
///
/// Парсер координат 3D модели из файла ДиалогСтатика
pub struct Import3DTanksEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl Import3DTanksEval {
    ///
    /// Новый экземпляр [Import3DTanksEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "Import3DTanksEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    ///
    /// Парсинг координат угловых точек переборок отсеков
    fn parsing_compartment_corner_points(&self, model_3d: Vec<f64>) -> CompartmentCornerPoints {
        let mut surface_outer_body_coords: Vec<Vec<(f64, f64, f64)>> = Vec::new();
        let mut i = 0;
        let mut flag = 0;
        while i < model_3d.len() - 3 {
            if flag == 1 { break; }
            if model_3d[i] == 999.0 {
                let mut current_frame = model_3d[i + 1];
                let mut current_vertices = Vec::new();
                i += 1;
                loop {
                    if i + 2 >= model_3d.len() {
                        flag += 1;
                        break;
                    }
                    let z = model_3d[i + 1];
                    if z == 999.0 {
                        if !current_vertices.is_empty() {
                            surface_outer_body_coords.push(current_vertices.clone());
                        }
                        if i + 2 >= model_3d.len() {
                            flag += 1;
                            break;
                        }
                        if model_3d[i + 2] == -999.0 {
                            flag += 1;
                            break;
                        }
                        current_frame = model_3d[i + 2];
                        current_vertices = Vec::new();
                        i += 2;
                        continue;
                    }
                    let y = model_3d[i + 2];
                    if y == 999.0 {
                        if !current_vertices.is_empty() {
                            surface_outer_body_coords.push(current_vertices.clone());
                        }
                        if i + 3 >= model_3d.len() {
                            flag += 1;
                            break;
                        }
                        if model_3d[i + 3] == -999.0 {
                            flag += 1;
                            break;
                        }
                        current_frame = model_3d[i + 3];
                        current_vertices = Vec::new();
                        i += 3;
                        continue;
                    }
                    current_vertices.push((current_frame, z, y));
                    i += 2;
                    if i >= model_3d.len() {
                        flag += 1;
                        break;
                    }
                }
            }
            i += 1;
        }
        return CompartmentCornerPoints { coordinates: surface_outer_body_coords };
    }
}
//
//
impl Eval<Zg, EvalResult> for Import3DTanksEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let path_3d_tanks = ContextReadRef::<InitialCtx>::read_ref(&ctx).path_3d_tanks.clone();
                let bytes = fs::read(path_3d_tanks)
                    .expect("Failed to read file");
                let tanks_3d = String::from_utf8_lossy(&bytes);
                let filtered_coords: Vec<f64> = tanks_3d
                .split([',', '\n', '\r', ' ', 'H', '\'', '�', '�', '�', '�'])
                .map(|s| s.trim())
                .filter(|s| {
                    !s.is_empty()
                })
                .filter_map(|s| 
                    s.parse::<f64>().ok()
                )
                .collect();
                let compartment_corner_points = self.parsing_compartment_corner_points(filtered_coords);
                ctx.write(Import3DTanksCtx {
                    compartment_corner_points: compartment_corner_points,
                })
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for Import3DTanksEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Import3DTanksEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}