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
/// Парсер координат 3D отсеков из файла ДиалогСтатика
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
    fn parsing_compartment_corner_points(&self, model_3d: &Vec<f64>) -> (Vec<CompartmentCornerPoints>, usize) {
        let mut result: Vec<CompartmentCornerPoints> = Vec::new();
        let mut i = 0;
        while i < model_3d.len() - 2 {
            if model_3d[i] == -999.0 {
                break;
            }
            else if model_3d[i] == 999.0 {
                if model_3d[i + 1] == -999.0 {
                    break;
                }
                let current_id = model_3d[i + 1];
                let x1 = model_3d[i + 2];
                let start = i + 2;
                let mut current_vertices: Vec<f64> = Vec::new();
                i += 3;
                while model_3d[i] != 999.0 {
                    current_vertices.push(model_3d[i]);
                    i += 1;
                }
                let end = i;
                let pair_count = (end - start) / 2 ; // сколько пар (z,y) приходится на x
                let mut curr_comp_pts = CompartmentCornerPoints::new();
                curr_comp_pts.id = current_id;
                // пары для x1
                let mut tmp = Vec::new();
                for i in (0..pair_count - 1).step_by(2) {
                    tmp.push(
                        (
                            current_vertices[i + 1],
                            current_vertices[i]
                        )
                    );
                }
                curr_comp_pts.coordinates_x1 = (x1, tmp.clone());
                // пары для x2
                let x2 = current_vertices[pair_count - 1];
                let mut tmp = Vec::new();
                for i in (pair_count..current_vertices.len()).step_by(2) {
                    tmp.push(
                        (
                            current_vertices[i + 1],
                            current_vertices[i]
                        )
                    );
                }
                curr_comp_pts.coordinates_x2 = (x2, tmp.clone());
                result.push(curr_comp_pts);
            }
        }
        return (result, i);
    }
    ///
    /// Парсинг названий отсеков 
    fn parsing_compartment_names(&self, position: usize, model_3d: &Vec<f64>) -> Vec<f64> {
        let mut result = Vec::new();
        let mut i = position + 2;
        while i < model_3d.len() - 2 {
            if model_3d[i] < 0.0 && model_3d[i] != -999.0 {
                result.push(model_3d[i]);
            }
            i += 1;
            // if model_3d[i + 1] == -999.0 && model_3d[i + 2] == -999.0 { // конец блока
            //     break;
            // } else if model_3d[i + 1] == -999.0 { // конец подблока
            //     i += 2;
            //     continue;
            // } else if model_3d[i] == 999.0 {
            //     if model_3d[i + 1] == -999.0 && model_3d[i + 2] == -999.0 { // конец блока
            //         break;
            //     } else if model_3d[i + 1] == -999.0 { // конец подблока
            //         i += 2;
            //         continue;
            //     }
            //     result.push(model_3d[i + 1]);
            //     println!("{:?}", model_3d[i + 1]);
            //     i += 2;
            // } else { // порядковый номер отсека
            //     result.push(model_3d[i + 1]);
            //     println!("{:?}", model_3d[i + 1]);
            //     i += 2;
            // }
        }
        result
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
                let  (mut compartment_corner_points, postion) = self.parsing_compartment_corner_points(&filtered_coords);
                let  compartment_id_to_reverse = self.parsing_compartment_names(postion, &filtered_coords);
                compartment_corner_points.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
                ctx.write(
                    Import3DTanksCtx {
                        compartment_corner_points,
                        compartment_id_to_reverse
                    }
                )
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