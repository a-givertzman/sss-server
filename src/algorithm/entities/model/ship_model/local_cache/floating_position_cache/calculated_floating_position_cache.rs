#[cfg(test)]
#[path = "../../../../../../../tests/unit/algorithm/models/ship_model/local_cache/floating_position_cache/calculated_floating_position_cache_test.rs"]
mod tests;
//
use sal_3dlib::{
    gmath::vector::Vector,
    props::{Center, Volume},
    topology::shape::{
        compound::{AlgoMakerVolume, Compound, Solids},
        face::{Face, Rotate, Translate},
        Shape,
    },
};
use sal_sync::services::{
    entity::{dbg_id::DbgId, error::str_err::StrErr},
    service::service_handles::ServiceHandles,
};
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
///
/// Provides logic to calculate and store cache used by [super::FloatingPositionCache].
///
/// See [super::FloatingPositionCacheConf] for more details about the fields.
pub(super) struct CalculatedFloatingPositionCache<A> {
    dbgid: DbgId,
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
        parent: &DbgId,
        file_path: PathBuf,
        elements: Vec<Shape<A>>,
        waterline: Face<A>,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbgid: DbgId::with_parent(parent, "CalculatedFloatingPositionCache"),
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
    pub(super) fn build(self) -> Result<ServiceHandles<Result<(), StrErr>>, StrErr>
    where
        A: Send + 'static,
    {
        let dbgid = DbgId(format!("{}.build", self.dbgid));
        log::info!("{} | Starting...", dbgid);
        match thread::Builder::new()
            .name(self.dbgid.0.clone())
            .spawn(move || self.calculate())
        {
            Ok(handler) => {
                log::info!("{} | Starting - OK", dbgid);
                Ok(ServiceHandles::new(vec![(dbgid.0, handler)]))
            }
            Err(why) => {
                let err_msg = format!("{} | Starting - FAILED: {}", dbgid, why);
                log::warn!("{}", err_msg);
                Err(StrErr(err_msg))
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
    fn calculate(self) -> Result<(), StrErr> {
        let dbgid = DbgId(format!("{}.calculate", self.dbgid));
        let out_f = &mut File::create(&self.file_path).map_err(|err| {
            StrErr(format!(
                "{} | Creating file='{}': {}",
                dbgid,
                self.file_path.display(),
                err
            ))
        })?;
        for &draught in &self.draught_steps {
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        log::warn!("{} | Interrupted: `exit` has got true", dbgid);
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
                                    Err(StrErr(format!(
                                        "{} | Triple [{}, {}, {}] gives no solids, but the volume={}.", 
                                        dbgid, heel, trim, draught, volume
                                    )))
                                } else {
                                    log::warn!(
                                        "{} | Triple [{}, {}, {}] gives no solids under the waterline.", 
                                        dbgid, heel, trim, draught
                                    );
                                    Ok(())
                                }
                            }
                            Some([x, y, z]) => writeln!(
                                out_f,
                                "{} {} {} {} {} {} {}",
                                heel, trim, draught, volume, x, y, z
                            )
                            .map_err(|err| {
                                StrErr(format!(
                                    "{} | Writing to file='{}': {}",
                                    dbgid,
                                    self.file_path.display(),
                                    err
                                ))
                            }),
                        })?;
                }
            }
        }
        Ok(())
    }
}
