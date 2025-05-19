//
use sal_3dlib::{
    gmath::vector::Vector,
    props::{Center, Volume},
    topology::shape::{
        Shape,
        compound::{AlgoMakerVolume, Compound, Solids},
        face::{Face, Rotate, Translate},
    },
};
use std::thread::JoinHandle;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::service::ServiceHandles;
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
///
/// Provides logic to calculate and store cache used by [super::FloatingPositionCache].
///
/// See [super::FloatingPositionCacheConf] for more details about the fields.
pub(super) struct CalculatedFloatingPositionCache<A> {
    dbg: Dbg,
    file_path: PathBuf,
    elements: Vec<Shape<A>>,
    waterline: Face<A>,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    ///
    /// Used to stop started worker thread.
    ///
    /// See [CalculatedFloatingPositionCache::calculate] for details.
    exit: Arc<AtomicBool>,
}
//
//
impl<A: Clone> CalculatedFloatingPositionCache<A> {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        file_path: PathBuf,
        elements: Vec<Shape<A>>,
        waterline: Face<A>,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "CalculatedFloatingPositionCache"),
            file_path,
            elements,
            waterline,
            heel_steps,
            trim_steps,
            draught_steps,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [FloatingPositionCache::calculate].
    pub(super) fn build(self) -> Vec<Error>
    where
        A: Send + 'static,
    {
        log::info!("{} | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut binding = File::create(&self.file_path).map_err(|err| {
            error.pass_with(
                format!("File::create error! path:{}", self.file_path.display()),
                err.to_string(),
            )
        });
        let mut out_f = match &mut binding {
            Ok(file) => file, 
            Err(err) => {
                return vec![error.pass_with(format!("File::create, path: {:?}", self.file_path), err.to_string())];
            },
        };
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let mut spawn_errors = Vec::new();
        for &draught in &self.draught_steps {
            let draught = draught*1000.;
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        log::warn!("{} | Interrupted: `exit` has got true", &self.dbg);
                        return Vec::<Error>::new();
                    }
                    let mut obj = self.waterline.clone();
                    let elements = self.elements.clone();
                    let task = thread::Builder::new().name(format!("self.dbg.to_string() task {heel} {trim} {draught}"))
                        .spawn(move || {
                            // make a clone of origin waterline and transform it
                            // according to heel, trim, and draught values
                            let obj = &{
                                let origin = obj.center();
                                let mut loc_y = Vector::unit_y();
                                if 0.0 != heel {
                                    let heel_in_rad = heel.to_radians();
                                    obj = obj.rotate(origin.clone(), Vector::unit_x(), heel_in_rad);
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
                            elements
                                .iter()
                                .filter_map(|elmnt| {
                                    // get compound as result of volume algorithm
                                    // applied to waterline and each target element
                                    // (taking into account its shape type)
                                    Some(match elmnt {
                                        Shape::Shell(elmnt) => {
                           //                 dbg!("elmnt ", elmnt.center().point());
                                            Compound::build([obj], [elmnt], [])
                                        },
                                        Shape::Solid(elmnt) => {
                            //                dbg!("elmnt ", elmnt.center().point());
                                            Compound::build([obj], [], [elmnt])
                                        },
                                        _ => return None,
                                    })
                                })
                                .try_fold((0.0, None), |(mut volume, mut volume_moment, ), build| {
                                    build.map(|volumed| {
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
                                                        log::error!("CalculatedFloatingPositionCache task: Compound::build volume error: {err}");
                                                    },
                                                }*/
                                            }
                                        });
                                  //      let text = format!("build.map volume:{} volume_moment:{:?}", volume, volume_moment);
                                  //      dbg!(text);
                                        (volume, volume_moment)
                                    })
                                })
                                .map(|(volume, volume_moment)|  {
                                    let volume_center = volume_moment
                                        .map(|[x, y, z]| [x/(volume*1000.), y/(volume*1000.), z/(volume*1000.)]);
                                    let volume = volume/1000000000.; //mm^3 to m^3
                                    let draught = draught/1000.; // mm to m
                               //     let text = format!("map, volume:{volume}, volume_center:{:?}, heel:{heel}, trim:{trim}, draught:{draught}", volume_center);
                               //     dbg!(text);
                                    (volume, volume_center, heel, trim, draught)
                                })
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
                    match task {
                        Ok(task) => tasks.push(task),
                        Err(err) => spawn_errors.push(err),
                    };                    
                }
            }
        }
        let errors = tasks
            .into_iter()
            .map(|task| match task.join() {
                Ok(res) => match res {
                    Ok((volume, mb_volume_center, heel, trim, draught)) => match mb_volume_center {
                        None => {
                            if volume > 0.0 {
                                Err(format!(
                                    "{} | no mb_volume_center, volume > 0.0, res={:?}.",
                                    &self.dbg, res
                                )
                                .into())
                            } else {
                                log::warn!(
                                    "{} | no solids, volume <= 0.0, res={:?}.",
                                    &self.dbg,
                                    res
                                );
                                Ok(())
                            }
                        }
                        Some([x, y, z]) => writeln!(
                            &mut out_f,
                            "{} {} {} {} {} {} {}",
                            heel, trim, draught, volume, x, y, z
                        )
                        .map_err(|err| {
                            error.pass_with(
                                format!("Writing to file, path:{}", self.file_path.display()),
                                err.to_string(),
                            )
                        }),
                    },
                    Err(err) => Err(error.pass_with(format!("work"), err.to_string())),
                },
                Err(_) => Err(error.err(format!("task.join"))),
            })
            .filter_map(|v| v.err())
            .collect();
        errors
    }
}
