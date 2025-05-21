use coco::Stack;
//
use sal_3dlib::{
    gmath::vector::Vector, ops::transform::*, props::{Center, Volume}, topology::shape::{
        compound::{AlgoMakerVolume, Compound, Solids}, face::*, Shape
    }
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{JoinHandle, Scheduler};
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::algorithm::entities::{model::ShipModelMeta, Position};
///
/// Provides logic to calculate and store cache used by [super::DisplacementCache].
///
/// See [super::DisplacementCacheConf] for more details about the fields.
// 
// normalize meters to/from mm
const SCALE: f64 = 1e3;

pub struct BuildDisplacementCache {
    dbg: Dbg,
    elements: Vec<Shape<ShipModelMeta>>,
    waterline_position: Position,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildDisplacementCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        elements: Vec<Shape<ShipModelMeta>>,
        waterline_position: Position,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildDisplacementCache"),
            elements,
            waterline_position,
            heel_steps,
            trim_steps,
            draught_steps,
            scheduler,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [DisplacementCache::calculate].
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let origin = self.waterline_position.scale(SCALE).into();
        let size = 1000.*SCALE;
        let rect = Vector::new(  size, size, size);;
        let mut waterline: Face<ShipModelMeta> = Workplane::xy().translated(origin).rect(&rect).to_face();
        'draught: for &draught in &self.draught_steps {
            let draught = draught*SCALE;
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        break 'draught;
                    }
                    let mut obj = waterline.clone();
                    let elements = self.elements.clone();
                    let dbg_ = self.dbg.clone();
                    let task_results = task_results.clone();
                    let handle = self.scheduler.spawn(move || {
                        // make a clone of origin waterline and transform it
                        // according to heel, trim, and draught values
                        let error = Error::new(&dbg_, format!("task {heel} {trim} {draught}"));
                        let obj = &{
                            let mut loc_y = Vector::unit_y();
                            if 0.0 != heel {
                                let heel_in_rad = heel.to_radians();
                                obj = obj.rotate(origin, Vector::unit_x(), heel_in_rad);
                                // once a rotation around oX happens, oY needs to get the rotation too,
                                // overwise oY remains global and doesn't match new `obj`'s transformation
                                loc_y = loc_y.rotate(Vector::unit_x(), heel_in_rad);
                            }
                            if 0.0 != trim {
                                obj = obj.rotate(origin, loc_y, trim.to_radians());
                            }
                            if 0.0 != draught {
                                obj = obj.translate(Vector::new(0.0, 0.0, draught));
                            }
                            obj
                        };
                        let mut volume = 0.0;
                        let mut volume_moment = None;
                        for elmnt in elements {
                            let volumed = match elmnt {
                                Shape::Shell(elmnt) => {
                                //                 dbg!("elmnt ", elmnt.center().point());
                                    Compound::build([obj], [&elmnt], [])
                                },
                                Shape::Solid(elmnt) => {
                                //                dbg!("elmnt ", elmnt.center().point());
                                    Compound::build([obj], [], [&elmnt])
                                },
                                _ => continue,
                            };
                            match volumed {
                                Ok(volumed) => {
                                    let solids: Vec<_> = volumed.solids().into_iter().collect();
                                    //   let text = format!("try_fold build draught:{} solids:{}", draught, solids.len());  
                                    //   dbg!(text);   

                                    solids.iter().for_each(|elmnt| {
                                        let [.., elmnt_z] = elmnt.center().point();
                                        let [.., waterline_z] = obj.center().point();
                                        // Only calculate volume if volumed element is below waterline.
                                        // Put 0.0 if it's not for consistent.
                                        //     dbg!("try_fold volumed", elmnt_z, waterline_z);
                                        //     let text = format!("try_fold volumed draught:{} elmnt_z:{} waterline_z:{}", draught, elmnt_z, waterline_z);  
                                        //     dbg!(text);   
                                        if elmnt_z < waterline_z { 
                                            //            let text = format!("try_fold volumed elmnt_z:{} < waterline_z elmnt_z:{}", elmnt_z, waterline_z);  
                                            //           dbg!(text);                                             
                                        /*     match Compound::build([obj], [], [elmnt]) {
                                                Ok(volumed) => {
                                                    volumed.solids().into_iter().for_each(|elmnt| {*/
                                                        let current_volume = elmnt.volume();
                                                        let [e_x, e_y, e_z] = elmnt.center().point();
                                                        let current_moment = [e_x*current_volume, e_y*current_volume, e_z*current_volume];
                                                        volume += current_volume;
                                                        match volume_moment.as_mut() {
                                                            None => volume_moment = Some(current_moment),
                                                            Some(volume_moment) => {
                                                                volume_moment[0] += current_moment[0];
                                                                volume_moment[1] += current_moment[1];
                                                                volume_moment[2] += current_moment[2];
                                                            }
                                                        }
                                                        let text = format!("volumed current_volume:{} center:{:?}", current_volume, elmnt.center().point());
                                                        dbg!(text);
                                            /*        });
                                                },
                                                Err(err) => {
                                                    log::error!("BuildDisplacementCache task: Compound::build volume error: {err}");
                                                },
                                            }*/
                                        }
                                    });
                                }
                                Err(_) => todo!(),
                            }
                            ///
                            /// - Transforms units
                            /// - Moment tranforms into Center of displaced valume (Mass center)
                            fn tranform(heel: f64, trim: f64, draught: f64, volume: f64, volume_moment: Option<[f64; 3]>) -> (f64, Option<[f64; 3]>, f64, f64, f64) {
                                let volume_center = volume_moment
                                    .map(|[x, y, z]| [x/(volume*SCALE), y/(volume*SCALE), z/(volume*SCALE)]);
                                let volume = volume / (SCALE*SCALE*SCALE); //mm^3 to m^3
                                let draught = draught / SCALE; // mm to m
                            //     let text = format!("map, volume:{volume}, volume_center:{:?}, heel:{heel}, trim:{trim}, draught:{draught}", volume_center);
                            //     dbg!(text);
                                (volume, volume_center, heel, trim, draught)
                            }
                            task_results.push(
                                tranform(heel, trim, draught, volume, volume_moment),
                            );
                        }
                        Ok(())
                    })
                    .map_err(|err| {
                        error.pass_with(
                            format!(
                                "spawn task draught:{} heel:{} trim:{}",
                                draught, heel, trim
                            ),
                            err.to_string(),
                        )
                    });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => results.push(Err(err)),
                    };                    
                }
            }
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                results.push(Err(error));
            }
        }
        while !task_results.is_empty() {
            if let Some((volume, mb_volume_center, heel, trim, draught)) = task_results.pop() {
                match mb_volume_center {
                    None => {
                        if volume > 0.0 {
                            results.push(Err(error.err(format!(
                                "no mb_volume_center, volume > 0.0, heel: {}, trim: {}, draught: {}",
                                heel, trim, draught,
                            ))))
                        } else {
                            log::warn!(
                                "{} | no solids, volume <= 0.0, heel: {}, trim: {}, draught: {}",
                                &self.dbg, heel, trim, draught,
                            );
                        }
                    }
                    Some([x, y, z]) => {
                       results.push(Ok(vec![heel, trim, draught, volume, x, y, z]));
                    }
                }
            }
        }
        results
    }
}
