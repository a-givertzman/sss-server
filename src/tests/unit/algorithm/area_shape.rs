#[cfg(test)]

mod tests {
    use crate::algorithm::entities::model_cached::AreaShape;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use nalgebra::{Point3, Vector3};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    #[ignore = "too slow, run only in release mode"]
    #[test]
    fn shape_windage_area() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test shape_aabb";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own("test shape_aabb");

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (position, objects) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            position.clone(),
            objects.clone(),
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.01;
        let mut shape = AreaShape::new(&dbg, mesh, None, None, Some(Point3::new(1., 0., 0.)), 1., 2000, None, None);
        shape._voxelize().unwrap();
        let result: f64 = shape.windage_area_data(-1.).unwrap().iter().map(|(_dx, area)| *area).sum();
        let target = 1.0;
        assert!(
            (result - target).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        test_duration.exit();
    }
}
