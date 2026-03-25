#[cfg(test)]

mod tests {
    use crate::algorithm::entities::model_cached::AreaShape;
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use nalgebra::Vector3;
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    //  #[ignore = "too slow, run only in release mode"]
    #[test]
    fn shape_windage_area() {
        DebugSession::new()
            .filter(LogLevel::Debug)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
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
        let mut shape = AreaShape::new(
            &dbg,
            mesh,
            None,
            None,
            Some(1.),
            1.,
            2000,
            None,
            Some(0.01),
            None,
        );
        shape._voxelize().unwrap();
        let result: f64 = shape
            .windage_area_data()
            .unwrap()
            .voxels
            .iter()
            .map(|(_dx, area)| area.iter().map(|&v| v.1).sum::<f64>())
            .sum();
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
