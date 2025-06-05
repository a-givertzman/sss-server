use coco::Stack;
use glam::DVec3;
use opencascade::primitives::{IntoShape, Shape};
//
/*
use sal_3dlib::{
    gmath::vector::Vector, ops::{transform::*, Polygon}, props::{Center, Volume}, topology::shape::{
        compound::{AlgoMakerVolume, Compound, Solids}, face::*, vertex::Vertex, wire::Wire, Shape
    }
};*/
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

use crate::algorithm::entities::Position;
///
/// Provides logic to calculate and store cache used by [super::DisplacementCache].
///
/// See [super::DisplacementCacheConf] for more details about the fields.
// 

pub struct BuildDisplacementCache {
    dbg: Dbg,
    shape: Arc<Shape>,
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
impl BuildDisplacementCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Shape,
        center_coord: Position,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
        scale: f64,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildDisplacementCache"),
            shape: Arc::new(shape),
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
    /// Creates and starts worker for [DisplacementCache::calculate].
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut results = Vec::new();
      //  let mut waterline: Face<ShipModelMeta> = Workplane::xy().translated(origin).rect(&rect).to_face();
        'draught: for &draught in &self.draught_steps {
            let draught = draught*self.scale;
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        break 'draught;
                    }
                  //  let dbg_ = self.dbg.clone();
                    let task_results = task_results.clone();
                    let handle = self.scheduler.spawn( move || {
                        // make a clone of origin waterline and transform it
                        // according to heel, trim, and draught values
                       // let error = Error::new(&dbg_, format!("task {heel} {trim} {draught}"));

                        let (volume, volume_center) = Self::calc_volume(
                            &self.shape,
                            self.scale.clone(),
                            self.center_coord.clone(),
                            heel,
                            trim,
                            draught,
                        );

                        task_results.push(
                            (volume, volume_center, heel, trim, draught),
                        );

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
                    results.push(Ok(vec![heel, trim, draught, volume, mb_volume_center.x(), mb_volume_center.y(), mb_volume_center.z()]));
            }
        }
     //   dbg!(&results);
        results
    }

    fn calc_volume(
        body: &Shape,
        scale_from_m: f64,
        center_coord: Position,
        heel: f64,
        trim: f64,
        draught: f64,
    ) -> (f64, Position) {
    //   let body = body.scale(DVec3::ZERO, 1. / scale_from_m);
        // центр для построения сечения, через эту точку должна проходить ватерлиния
        let origin = DVec3::new(center_coord.x()*scale_from_m, center_coord.y()*scale_from_m, (center_coord.z() + draught)*scale_from_m);
        // строим коробку с центром в (0, 0, 0), которая будет отсекать погруженную в воду часть модели
        let half_size = 1000.*scale_from_m;
        let corner_1 = DVec3::new(half_size, half_size, half_size);
        let corner_2 = DVec3::new(-half_size, -half_size, -half_size);
        let mut water_box = Shape::box_from_corners(corner_1, corner_2);
        water_box.translate(DVec3::new(origin.x, origin.y, origin.z - half_size));
        if heel != 0. {
            water_box = water_box.rotate(origin, DVec3::X, -heel);
        }
        if trim != 0. {
            water_box = water_box.rotate(origin, DVec3::Y, -trim);
        }
        let intersect = body.intersect(&water_box);
        let shape = intersect.into_shape();
        // let time = Instant::now();
        let (volume, center) = shape.volume_data();
        //  let time_volume_data = time.elapsed();
        (volume/(scale_from_m*scale_from_m*scale_from_m), Position::new(center.x/scale_from_m - center_coord.x(), -center.y/scale_from_m - center_coord.y(), center.z/scale_from_m - center_coord.z()))
    }
}
