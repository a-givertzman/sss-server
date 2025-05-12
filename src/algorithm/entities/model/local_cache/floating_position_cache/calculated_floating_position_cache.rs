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
    pub(super) fn build(self) -> Result<ServiceHandles<Result<(), Error>>, Error>
    where
        A: Send + 'static,
    {
        let dbg = Dbg::new(&self.dbg, "build");
        log::info!("{} | Starting...", dbg);
        match thread::Builder::new()
            .name(self.dbg.to_string())
            .spawn(move || self.calculate())
        {
            Ok(handler) => {
                log::info!("{} | Starting - OK", dbg);
                Ok(ServiceHandles::new(vec![(dbg.to_string(), handler)]))
            }
            Err(why) => {
                let err_msg = format!("{} | Starting - FAILED: {}", dbg, why);
                log::warn!("{}", err_msg);
                Err(Error::new(dbg, "build").pass(err_msg))
            }
        }
    }
    ///
    /// Builds the cache and stores it into `self.file_path`.
    ///
    /// The caller can stop executing by setting `self.exit` to _true_.
    ///
    /// While calculating it iterates over `self.heel_steps`, `self.trim_steps`,
    /// and `self.draught_steps` to set a new position to cloned `self.waterline`.
    /// The cloned waterline is used to apply volume algorithm to `self.models`,
    /// to get, in order, _volume_ of all volumed parts placed under the waterline.
    /// At the end of each iteration, a line is written to the output file in format:
    /// "{heel_step} {trim_step} {draught_step} {volume}".
    fn calculate(self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "calculate");
        let out_f = &mut File::create(&self.file_path).map_err(|err| {
            error.pass_with(
                format!("File::create error! path:{}", self.file_path.display()),
                err.to_string(),
            )
        })?;
        for &draught in &self.draught_steps {
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        log::warn!("{} | Interrupted: `exit` has got true", &self.dbg);
                        return Ok(());
                    }
                    // make a clone of origin waterline and transform it
                    // according to heel, trim, and draught values
                    let w_obj = &{
                        let mut obj = self.waterline.clone();
                        let origin = self.waterline.center();
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
                            obj = obj.translate(Vector::new(0.0, 0.0, -draught));
                        }
                        obj
                    };
                    self.elements
                        .iter()
                        .filter_map(|elmnt| {
                            // get compound as result of volume algorithm
                            // applied to waterline and each target element
                            // (taking into account its shape type)
                            Some(match elmnt {
                                Shape::Shell(elmnt) => Compound::build([w_obj], [elmnt], []),
                                Shape::Solid(elmnt) => Compound::build([w_obj], [], [elmnt]),
                                _ => return None,
                            })
                        })
                        .try_fold((0.0, None), |(mut volume, mut mb_volume_center), build| {
                            build.map(|volumed| {
                                volumed.solids().into_iter().for_each(|elmnt| {
                                    let [.., elmnt_z] = elmnt.center().point();
                                    let [.., waterline_z] = w_obj.center().point();
                                    // Only calculate volume if volumed element is below waterline.
                                    // Put 0.0 if it's not for consistent.
                                    if elmnt_z < waterline_z {
                                        volume += elmnt.volume();
                                        match mb_volume_center.as_mut() {
                                            None => mb_volume_center = Some(elmnt.center().point()),
                                            Some(center) => {
                                                let [e_x, e_y, e_z] = elmnt.center().point();
                                                center[0] += e_x;
                                                center[1] += e_y;
                                                center[2] += e_z;
                                            }
                                        }
                                    }
                                });
                                (volume, mb_volume_center)
                            })
                        })
                        .and_then(|(volume, mb_volume_center)| match mb_volume_center {
                            None => {
                                if volume > 0.0 {
                                    Err(
                                        format!("{} | Triple [{}, {}, {}] gives no solids, but the volume={}.",
                                        &self.dbg, heel, trim, draught, volume).into()
                                    )
                                } else {
                                    log::warn!(
                                        "{} | Triple [{}, {}, {}] gives no solids under the waterline.", 
                                        &self.dbg, heel, trim, draught
                                    );
                                    Ok(())
                                }
                            },
                            Some([x, y, z]) => {
                                Ok(writeln!(
                                    out_f,
                                    "{} {} {} {} {} {} {}",
                                    heel, trim, draught, volume, x, y, z
                                )
                                .map_err(|err| {
                                    format!(
                                        "{} | Writing to file, error:{} path:{}",
                                        &self.dbg, err.to_string(), self.file_path.display()
                                    )
                                })?)
                            },
                        });
                }
            }
        }
        Ok(())
    }
}
