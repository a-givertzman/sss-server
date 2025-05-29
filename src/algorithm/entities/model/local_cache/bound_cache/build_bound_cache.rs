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
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::algorithm::entities::{Position, model::ShipModelMeta};
///
/// Provides logic to calculate and store cache used by [super::BoundCache].
///
/// See [super::BoundCacheConf] for more details about the fields.
//

pub struct BuildBoundCache {
    dbg: Dbg,
    elements: Vec<Shape<ShipModelMeta>>,
    waterline_position: Position,
    heel_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
    scale: f64,
}
//
//
impl BuildBoundCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        elements: Vec<Shape<ShipModelMeta>>,
        waterline_position: Position,
        heel_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
        scale: f64,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildBoundCache"),
            elements,
            waterline_position,
            heel_steps,
            draught_steps,
            scheduler,
            exit,
            scale,
        }
    }
    ///
    /// Creates and starts worker for [BoundCache::calculate].
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let origin = self.waterline_position.scale(self.scale);
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
                            // translate сбрасывает вращение, поэтому сначала перемещаем, потом вращаем
                            obj = obj.translate(Vector::new(0.0, 0.0, draught));
                            if 0.0 != heel {
                                let heel_in_rad = heel.to_radians();
                                obj = obj.rotate(origin_fixed.clone(), Vector::unit_x(), heel_in_rad);
                            }
                            obj
                        };
                        let mut volume = 0.0;
                        for elmnt in elements {
                            let volumed = match elmnt {
                                Shape::Shell(elmnt) => Compound::build([obj], [&elmnt], []),
                                Shape::Solid(elmnt) => Compound::build([obj], [], [&elmnt]),
                                _ => continue,
                            };
                            match volumed {
                                Ok(volumed) => {
                                    let solids: Vec<_> = volumed.solids().into_iter().collect();
                                    solids.iter().for_each(|elmnt| {
                                        let [.., elmnt_z] = elmnt.center().point();
                                        let [.., waterline_z] = obj.center().point();
                                        // Only calculate volume if volumed element is below waterline.
                                        // Put 0.0 if it's not for consistent.
                                        if elmnt_z < waterline_z {
                                            volume += elmnt.volume();
                                        }
                                    });
                                }
                                Err(_) => todo!(),
                            }
                            ///
                            /// - Transforms units
                            fn tranform(
                                heel: f64,
                                draught: f64,
                                volume: f64,
                                scale: f64,
                            ) -> (f64, f64, f64) {
                                let volume = volume / (scale * scale * scale);
                                let draught = draught / scale;
                                (volume, heel, draught)
                            }
                            task_results.push(tranform(heel, draught, volume, scale));
                        }
                        Ok(())
                    })
                    .map_err(|err| {
                        error.pass_with(
                            format!("spawn task draught:{} heel:{}", draught, heel),
                            err.to_string(),
                        )
                    });
                match handle {
                    Ok(task) => tasks.push(task),
                    Err(err) => results.push(Err(err)),
                };
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
            if let Some((volume, heel, draught)) = task_results.pop() {
                results.push(Ok(vec![heel, draught, volume]));
            }
        }
        //   dbg!(&results);
        results
    }
}
