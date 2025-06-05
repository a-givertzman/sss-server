use coco::Stack;
//
use sal_3dlib::{
    gmath::vector::Vector,
    ops::{Polygon, transform::*},
    props::{Center, Volume},
    topology::shape::{
        Shape,
        compound::{AlgoMakerVolume, Compound, Solids},
        face::*,
        vertex::Vertex,
        wire::Wire,
    },
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::{JoinHandle, Scheduler};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::algorithm::entities::{Position, model::ShipModelMeta};
///
/// Provides logic to calculate and store cache used by [super::CompartmentCache].
///
/// See [super::CompartmentCacheConf] for more details about the fields.
//

pub struct BuildCompartmentCache {
    dbg: Dbg,
    elements: Vec<Shape<ShipModelMeta>>,
    center_coord: Position,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
    scale: f64,
}
//
//
impl BuildCompartmentCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        elements: Vec<Shape<ShipModelMeta>>,
        center_coord: Position,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
        scale: f64,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildCompartmentCache"),
            elements,
            center_coord,
            heel_steps,
            trim_steps,
            draught_steps,
            scheduler,
            exit,
            scale,
        }
    }
    ///
    /// Creates and starts worker for [CompartmentCache::calculate].
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let origin = self.center_coord.scale(self.scale);
        let size = 1000. * self.scale;
        let waterline = match Wire::polygon(
            [
                Vertex::new([origin.x() + size, origin.y() + size, origin.z()]),
                Vertex::new([origin.x() - size, origin.y() + size, origin.z()]),
                Vertex::new([origin.x() - size, origin.y() - size, origin.z()]),
                Vertex::new([origin.x() + size, origin.y() - size, origin.z()]),
            ],
            true,
        ) {
            Ok(ref polygon) => Face::try_from(polygon)
                .map_err(|err| error.pass_with("Failed creating Face from *polygon*", err)),
            Err(err) => {
                Err(error.pass_with("Failed creating *polygon* from Wire", err.to_string()))
            }
        };
        let waterline = match waterline {
            Ok(v) => v,
            Err(err) => return vec![Err(err)],
        };
        //  let mut waterline: Face<ShipModelMeta> = Workplane::xy().translated(origin).rect(&rect).to_face();
        'draught: for &draught in &self.draught_steps {
            let draught = draught * self.scale;
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        break 'draught;
                    }
                    let mut obj = waterline.clone();
                    let elements = self.elements.clone();
                    //  let dbg_ = self.dbg.clone();
                    let task_results = task_results.clone();
                    // смещаем origin на осадку для фикса бага translate
                    let origin_fixed = Vertex::new([
                        origin.values()[0],
                        origin.values()[1],
                        origin.values()[2] + draught,
                    ]);
                    // let origin = Vertex::new(origin.values());
                    let scale = self.scale;
                    let handle = self
                        .scheduler
                        .spawn(move || {
                            // make a clone of origin waterline and transform it
                            // according to heel, trim, and draught values
                            // let error = Error::new(&dbg_, format!("task {heel} {trim} {draught}"));
                            let obj = &{
                                let mut loc_y = Vector::unit_y();
                                // translate сбрасывает вращение, поэтому сначала перемещаем, потом вращаем
                                obj = obj.translate(Vector::new(0.0, 0.0, draught));
                                if 0.0 != heel {
                                    let heel_in_rad = heel.to_radians();
                                    obj = obj.rotate(
                                        origin_fixed.clone(),
                                        Vector::unit_x(),
                                        heel_in_rad,
                                    );
                                    // once a rotation around oX happens, oY needs to get the rotation too,
                                    // overwise oY remains global and doesn't match new `obj`'s transformation
                                    loc_y = loc_y.rotate(Vector::unit_x(), heel_in_rad);
                                }
                                if 0.0 != trim {
                                    obj = obj.rotate(origin_fixed, loc_y, trim.to_radians());
                                }
                                /*      if 0.0 != draught {
                                    obj = obj.translate(Vector::new(0.0, 0.0, draught));
                                }*/
                                obj
                            };
                            let mut volume = 0.0;
                            let mut volume_moment = None;
                            for elmnt in elements {
                                let volumed = match elmnt {
                                    Shape::Shell(elmnt) => {
                                        //                 dbg!("elmnt ", elmnt.center().point());
                                        Compound::build([obj], [&elmnt], [])
                                    }
                                    Shape::Solid(elmnt) => {
                                        //                dbg!("elmnt ", elmnt.center().point());
                                        Compound::build([obj], [], [&elmnt])
                                    }
                                    _ => continue,
                                };
                                match volumed {
                                    Ok(volumed) => {
                                        let solids: Vec<_> = volumed.solids().into_iter().collect();
                                        //    let text = format!("try_fold build heel:{heel}, trim:{trim}, draught:{draught} solids:{}", solids.len());
                                        //    dbg!(text);
                                        solids.iter().for_each(|elmnt| {
                                            let [.., elmnt_z] = elmnt.center().point();
                                            let [.., waterline_z] = obj.center().point();
                                            // Only calculate volume if volumed element is below waterline.
                                            // Put 0.0 if it's not for consistent.
                                            //     dbg!("try_fold volumed", elmnt_z, waterline_z);
                                            //     let text = format!("try_fold volumed heel:{heel}, trim:{trim}, draught:{draught} elmnt_z:{} waterline_z:{}", elmnt_z, waterline_z);
                                            //     dbg!(text);
                                            if elmnt_z < waterline_z {
                                                //            let text = format!("try_fold volumed elmnt_z:{} < waterline_z elmnt_z:{} heel:{heel}, trim:{trim}, draught:{draught}", elmnt_z, waterline_z);
                                                //           dbg!(text);
                                                let current_volume = elmnt.volume();
                                                let [e_x, e_y, e_z] = elmnt.center().point();
                                                let current_moment = [
                                                    e_x * current_volume,
                                                    e_y * current_volume,
                                                    e_z * current_volume,
                                                ];
                                                volume += current_volume;
                                                match volume_moment.as_mut() {
                                                    None => volume_moment = Some(current_moment),
                                                    Some(volume_moment) => {
                                                        volume_moment[0] += current_moment[0];
                                                        volume_moment[1] += current_moment[1];
                                                        volume_moment[2] += current_moment[2];
                                                    }
                                                }
                                                //     let text = format!("volumed current_volume:{} center:{:?} heel:{heel}, trim:{trim}, draught:{draught}", current_volume, elmnt.center().point());
                                                //    dbg!(text);
                                            }
                                        });
                                    }
                                    Err(_) => todo!(),
                                }
                                ///
                                /// - Transforms units
                                /// - Moment tranforms into Center of displaced valume (Mass center)
                                fn tranform(
                                    heel: f64,
                                    trim: f64,
                                    draught: f64,
                                    volume: f64,
                                    volume_moment: Option<[f64; 3]>,
                                    scale: f64,
                                ) -> (f64, Option<[f64; 3]>, f64, f64, f64)
                                {
                                    let volume_center = volume_moment.map(|[x, y, z]| {
                                        [
                                            x / (volume * scale),
                                            y / (volume * scale),
                                            z / (volume * scale),
                                        ]
                                    });
                                    let volume = volume / (scale * scale * scale); //mm^3 to m^3
                                    let draught = draught / scale; // mm to m
                                    //     let text = format!("map, volume:{volume}, volume_center:{:?}, heel:{heel}, trim:{trim}, draught:{draught}", volume_center);
                                    //     dbg!(text);
                                    (volume, volume_center, heel, trim, draught)
                                }
                                task_results.push(tranform(
                                    heel,
                                    trim,
                                    draught,
                                    volume,
                                    volume_moment,
                                    scale,
                                ));
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
                                &self.dbg,
                                heel,
                                trim,
                                draught,
                            );
                        }
                    }
                    Some([x, y, z]) => {
                        results.push(Ok(vec![heel, trim, draught, volume, x, y, z]));
                    }
                }
            }
        }
        //   dbg!(&results);
        results
    }
}
