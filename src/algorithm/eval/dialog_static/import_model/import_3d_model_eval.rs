use sal_core::{dbg::Dbg, error::Error};
use core::panic;
use std::fs;
use crate::algorithm::eval::entities::diametrical_buttocks::DiametricalButtocks;
use crate::algorithm::eval::entities::surface_outer_body::SurfaceOuterBody;
use crate::algorithm::eval::entities::surface_superstructure::SurfaceSuperstructure;
use crate::algorithm::eval::import_model::import_3d_model_ctx::Import3DModelCtx;
use crate::main;
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
pub struct Import3DModelEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl Import3DModelEval {
    ///
    /// Новый экземпляр [Import3DModelEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "Import3DModelEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
    ///
    /// Парсинг диаметрального батокса 
    fn parsing_diametrical_buttocks(&self, model_3d: Vec<f64>) -> (usize, DiametricalButtocks, DiametricalButtocks) {
        let mut stern_block= DiametricalButtocks::new();
        let mut flag = 0;
        let mut nasal_block= DiametricalButtocks::new();
        let mut i = 0;
        while i < model_3d.len() - 3 {
            if flag == 2 { break; }
            if model_3d[i] == 999.0 {
                loop {
                    let z = model_3d[i+1];
                    let x = model_3d[i+2];
                    if z == 999.0 {
                        flag += 1;
                        break;
                    }
                    if flag == 0 {
                        stern_block.coordinates.push(
                            (
                                z,
                                x,
                            )
                        );
                    } else if flag == 1 {
                        nasal_block.coordinates.push(
                            (
                                z,
                                x,
                            )
                        );
                    }
                    i += 2;
                }
            }
            i += 1;
        }
        return (i, stern_block, nasal_block);
    }
    ///
    /// Парсинг поверхности наружного корпуса
    fn parsing_surface_outer_body(&self, position: usize, model_3d: Vec<f64>, main_decks_positions: Vec<usize>) -> (usize, SurfaceOuterBody) {
        let mut surface_outer_body_coords: Vec<Vec<(f64, f64, f64)>> = Vec::new();
        let mut main_decks: Vec<usize> = Vec::new();
        let mut i = position;
        let mut flag_main = false;
        let mut flag = 0;
        while i < model_3d.len() - 3 {
            if flag == 1 { break; }
            if model_3d[i] == -999.0 {
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
                            if flag_main == true {
                                main_decks.push(surface_outer_body_coords.len());
                                surface_outer_body_coords.push(current_vertices.clone());
                                flag_main = false
                            } else {
                                surface_outer_body_coords.push(current_vertices.clone());
                            }
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
                        if main_decks_positions.contains(&(i + 2)) {
                            flag_main = true;
                        } else { flag_main = false; }
                        current_vertices = Vec::new();
                        i += 2;
                        continue;
                    }
                    let y = model_3d[i + 2];
                    if y == 999.0 {
                        if !current_vertices.is_empty() {
                            if flag_main == true {
                                main_decks.push(surface_outer_body_coords.len());
                                surface_outer_body_coords.push(current_vertices.clone());
                                flag_main = false
                            } else {
                                surface_outer_body_coords.push(current_vertices.clone());
                            }
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
                        if main_decks_positions.contains(&(i + 3)) {
                            flag_main = true;
                        } else { flag_main = false; }
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
        return (
            i, 
            SurfaceOuterBody { 
                coordinates: surface_outer_body_coords,
                main_deck: main_decks
            }
        );
    }
    ///
    /// Парсинг поверхности надстройки
    fn parsing_surface_superstructure(&self, position: usize, model_3d: Vec<f64>) -> SurfaceSuperstructure {
        let mut surface_superstructure_coords: Vec<Vec<(f64, f64, f64)>> = Vec::new();
        let mut i = position;
        let mut flag = 0;
        while i < model_3d.len() - 3 {
            let mut current_frame = model_3d[i + 1];
            let mut current_vertices = Vec::new();
            i += 1;
            if flag == 1 { break; }
            loop {
                let z = model_3d[i + 1];
                if z == 999.0 {
                    if !current_vertices.is_empty() {
                        surface_superstructure_coords.push(current_vertices.clone());
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
                        surface_superstructure_coords.push(current_vertices.clone());
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
            }
            i += 1;
        }
        return SurfaceSuperstructure { coordinates: surface_superstructure_coords };
    }
}
//
//
impl Eval<Zg, EvalResult> for Import3DModelEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let path_3d_model = ContextReadRef::<InitialCtx>::read_ref(&ctx).path_3d_model.clone();
                let mut main_deck_indexes = Vec::new();
                let bytes = fs::read(path_3d_model)
                    .expect("Failed to read file");
                let model_3d = String::from_utf8_lossy(&bytes);
                let filtered_coords: Vec<f64> = model_3d
                .split([',', '\n', '\r', ' ', '\'', '�', '�', '�', '�'])
                .map(|s| s.trim())
                .filter(|s| {
                    !s.is_empty()
                })
                .enumerate()
                .filter_map(|(i, s)| {
                    if s == 'H'.to_string() {
                        main_deck_indexes.push(i - main_deck_indexes.len());
                    }
                    s.parse::<f64>().ok()
                }
                )
                .collect();
                let (position, stern_block, nasal_block) = self.parsing_diametrical_buttocks(filtered_coords.clone());
                let (position, surface_outer_body) = self.parsing_surface_outer_body(position, filtered_coords.clone(), main_deck_indexes);
                let surface_superstructure = self.parsing_surface_superstructure(position, filtered_coords);
                ctx.write(Import3DModelCtx {
                    stern_block: stern_block,
                    nasal_block: nasal_block,
                    surface_outer_body,
                    surface_superstructure: surface_superstructure,
                })
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for Import3DModelEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Import3DModelEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
