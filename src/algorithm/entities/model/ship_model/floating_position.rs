use super::local_cache::LocalCache;
use sal_3dlib::{
    gmath::{point::Point, vector::Vector},
    ops::boolean::{Intersect, OpConf},
    props::{Center, Length},
    topology::shape::{
        compound::Edges,
        edge::{Direction, Edge, Rotate as _, Translate as _},
        face::{Face, Normal, Project, Rectangle, Rotate as _, Translate as _},
        vertex::Vertex,
    },
};
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
//
//
pub struct EvaluatedFloatingPosition {
    pub heel_angle: f64,
    pub trim_angle: f64,
    pub draught_at_amidships: f64,
    pub displacement: f64,
    pub displacement_center: [f64; 3],
    pub displacement_volume: f64,
    pub displacement_volume_center: [f64; 3],
    pub water_density: f64,
    pub accuracy: f64,
}
//
//
pub struct FloatingPosition<'cache, Attr> {
    dbgid: DbgId,
    cache: &'cache dyn LocalCache,
    centreline: Edge<Attr>,
    middle: Face<Attr>,
    disp: f64,
    disp_center: Vertex<Attr>,
    density: f64,
    accuracy: f64,
}
//
//
#[allow(clippy::too_many_arguments)]
impl<'cache, Attr> FloatingPosition<'cache, Attr> {
    pub(super) fn new(
        parent: &DbgId,
        cache: &'cache dyn LocalCache,
        centreline: Edge<Attr>,
        middle: Face<Attr>,
        disp: f64,
        disp_center: Vertex<Attr>,
        density: f64,
        accuracy: f64,
    ) -> Self {
        Self {
            dbgid: DbgId::with_parent(parent, "FloatingPosition"),
            cache,
            centreline,
            middle,
            disp,
            disp_center,
            density,
            accuracy,
        }
    }
    ///
    /// Evaluates floating posistion.
    ///
    /// # Panics
    /// Panic occurs if cached dataset is inconsistent. In particular, `disp_vol_center`,
    /// which read from the cache, _must be_ a point in 3-dimensional space.
    pub fn eval(mut self) -> Result<EvaluatedFloatingPosition, StrErr>
    where
        Attr: Clone,
    {
        let dbgid = DbgId(format!("{}/eval", self.dbgid));
        let init_keel = self.centreline.center().point();
        let disp_vol = self.disp / self.density;
        let mut theta = 0.0;
        let mut psi = 0.0;
        loop {
            let (draught, disp_vol_center) = {
                // Prepare values (key) to extract data from `self.cache`.
                // Note that 3rd parameter sets to None (as well as 5th and the rest).
                // This means we expect to get their approximated values from the cache.
                let approx_vals = [
                    Some(theta),
                    Some(psi),
                    None, // seeking _draught_: f64
                    Some(disp_vol),
                    // ... seeking _disp_vol_center_ (~ CB): [f64; 3]
                ];
                // The cache returns the whole row(s) for given `approx_vals`
                // and we expect each row has at least the following structure:
                //
                //   | heel | trim | draught | disp_vol | (disp_vol_center)_x | ()_y | ()_z |
                //
                // where
                // - every value is of type f64,
                // - heel ~ theta,
                // - trim ~ psi.
                //
                self.cache
                    .get(&approx_vals)
                    .inspect(|rows| {
                        // Normally, _one row_ for given `approx_vals` is expected,
                        // but in case of many rows, a warning is written to the log,
                        // and the algorithm continues consuming _only the first_ row.
                        if rows.len() > 1 {
                            log::warn!(
                                "{} | More than one cached row for approx_vals='{:?}'",
                                dbgid,
                                approx_vals
                            );
                        }
                    })
                    .and_then(|rows| rows.into_iter().next())
                    .ok_or(format!(
                        "{} | No value found for approx_vals='{:?}'",
                        dbgid, approx_vals
                    ))
                    .map(|row| {
                        let mb_disp_vol_center = &row[4..=6];
                        if let [x, y, z] = *mb_disp_vol_center {
                            return (row[2], Vertex::new([x, y, z]));
                        }
                        // The cache is inconsistent in terms of length of columns.
                        panic!(
                            "{} |`center_of_displacement_volume` must be a point \
                                in 3-dimensional space, but it has {} coordinates",
                            dbgid,
                            mb_disp_vol_center.len()
                        );
                    })?
            };
            {
                let horizontal_dist = {
                    let [ax, ay, ..] = disp_vol_center.point();
                    let [bx, by, ..] = self.disp_center.point();
                    ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt()
                };
                if horizontal_dist < self.accuracy {
                    return Ok(EvaluatedFloatingPosition {
                        heel_angle: theta,
                        trim_angle: psi,
                        draught_at_amidships: draught,
                        displacement: self.disp,
                        displacement_center: self.disp_center.point(),
                        displacement_volume: disp_vol,
                        displacement_volume_center: disp_vol_center.point(),
                        water_density: self.density,
                        accuracy: self.accuracy,
                    });
                }
            }
            let [frac_delta_psi_2, frac_delta_theta_2] = {
                let cg = Point::from(self.disp_center.point());
                let size = self.centreline.len();
                let v_plane_normal = self.centreline.dir().cross(&Vector::unit_z());
                let v_plane =
                    Face::rect(&self.disp_center, &v_plane_normal, 0.5 * size, 1.5 * size);
                let frac_delta_psi_2 = 0.5 * {
                    let cb_v = v_plane
                        .project(&disp_vol_center /* ~ CB */)
                        .map(|vertex| Point::from(vertex.point()))?;
                    let cg_h = {
                        let [.., z] = *cb_v;
                        let [x, y, ..] = self.disp_center.point();
                        Point::from(Vertex::<Attr>::new([x, y, z]).point())
                    };
                    //           ^
                    //           Z
                    //           |
                    //     +--+--+
                    //     |4 |1 |  \
                    //     +--#--+   > part of vertical plane, where # points to CG,
                    //     |3 |2 |  /  split into possible sections
                    // <---+--+--+
                    //  \
                    //   centreline direction projected on horizontal plane
                    //
                    // If _delta psi_ is located in 1 or 3 section (see pic),
                    // its value becomes positive (by defenition of trim).
                    //
                    // Example for section 3:
                    //    .... # - CG
                    //    .   /|
                    //    .  /=|-- delta psi
                    //    . /  |
                    //    ./   |
                    // <--#----# - CG_H
                    // |   \
                    // |    CB_V
                    // |
                    //  centreline direction projected on horizontal plane
                    //
                    let sign = {
                        let [section_1, section_3] = {
                            let [.., cg_z] = *cg;
                            let [cb_v_x, ..] = *cb_v;
                            let [cg_h_x, _, cg_h_z] = *cg_h;
                            [
                                cb_v_x >= cg_h_x && cg_h_z >= cg_z,
                                cb_v_x <= cg_h_x && cg_h_z <= cg_z,
                            ]
                        };
                        if section_1 || section_3 {
                            1.0
                        } else {
                            -1.0
                        }
                    };
                    // calculate 'delta psi' (see schema in algorithm docs)
                    // taking into account its section (see `sign` comments)
                    sign * Vector::from([cg, cb_v]).angle(&Vector::from([cg, cg_h]))
                };
                let m_plane_normal = self.middle.normal_at(&self.middle.center());
                let frac_delta_theta_2 = 0.5 * {
                    let m_plane = {
                        let size = 0.5 * size;
                        Face::rect(&self.disp_center, &m_plane_normal, size, size)
                    };
                    let intersection = v_plane
                        .intersect(&m_plane, OpConf { parallel: true })
                        .edges()
                        .into_iter()
                        .next()
                        .as_ref()
                        .map(Edge::dir)
                        .ok_or(format!(
                            "{} | No intersection between Vertical\
                            plane and Parallel to Midlle planes.",
                            dbgid
                        ))?;
                    let cb_m = m_plane
                        .project(&disp_vol_center)
                        .map(|vertex| Point::from(vertex.point()))?;
                    // Consider the tail of the model is behind the drawn part, then:
                    //      |
                    //   +--+--+
                    //   |4 |1 |  \
                    //   +--#--+   > part of plane parallel to `self.middle`,
                    //   |3 |2 |  /  where # points to CG, split into possible sections
                    // --+--+--+--
                    // |    |
                    // |     edge, which is the result of plane parallel to `self.middle`
                    // |     and _vertical_ plane intersection
                    // |
                    //  edge, which is the result of plane parallel to `self.middle`
                    //  and _horizontal_ plane intersection
                    //
                    // If _delta theta_ is located in 2 or 4 section (see pic),
                    // its value becomes negative (by defenition of heel).
                    //
                    // Example for section 2:
                    // CG - #.....
                    //      |\   .
                    //      |=\--.-- delta theta
                    //      |  \ .
                    //      |   \.
                    //    --#----#-- - edge, which is the result of plane parallel to `self.middle`
                    //     /    /      and _horizontal_ plane intersection
                    //    /     CB_M
                    //   /
                    //  point of vertical plane, horizontal plane,
                    //  and plane parallel to `self.middle` intersection
                    //
                    let sign = {
                        let [section_2, section_4] = {
                            let [.., cg_y, cg_z] = *cg;
                            let [.., cb_m_y, cb_m_z] = *cb_m;
                            [
                                cg_y >= cb_m_y && cg_z >= cb_m_z,
                                cg_y <= cb_m_y && cg_z <= cb_m_z,
                            ]
                        };
                        if section_2 || section_4 {
                            -1.0
                        } else {
                            1.0
                        }
                    };
                    // calculate 'delta psi' (see schema in algorithm docs)
                    // taking into account the actual section (see `sign` comments)
                    sign * Vector::from([cg, cb_m]).angle(&intersection)
                };
                // apply rotations to `self.middle` and `self.centreline`
                {
                    let cg = Vertex::new(*cg);
                    self.middle = self
                        .middle
                        .rotate(cg.clone(), v_plane_normal, frac_delta_psi_2);
                    self.centreline = self
                        .centreline
                        .rotate(cg.clone(), v_plane_normal, frac_delta_psi_2)
                        .rotate(cg, m_plane_normal, frac_delta_theta_2);
                }
                [frac_delta_psi_2, frac_delta_theta_2]
            };
            // Align the current keel point vertiacally to its initial position.
            // This is necessary to get correct results from cache on next iterations.
            {
                let dir = {
                    let cur_keel = Point::from(self.centreline.center().point());
                    let new_keel = {
                        let [x, y, ..] = init_keel;
                        let [.., z] = *cur_keel;
                        Point::from([x, y, z])
                    };
                    Vector::from([cur_keel, new_keel])
                };
                self.middle = self.middle.translate(dir);
                self.centreline = self.centreline.translate(dir);
            }
            theta += frac_delta_theta_2;
            psi += frac_delta_psi_2;
        }
    }
}
