#[cfg(test)]

mod tests {
    use crate::algorithm::entities::{Position, model_cached::DisplacementShape};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use nalgebra::{Point3, Vector3};
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    //
    #[test]
    fn shape_volume() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test shape_volume";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own("test shape_volume");

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (position, objects) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            position.clone(),
            objects.clone(),
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(Point3::new(1., 0., 0.)), 1., 0.0000001, 1000);
        let (result, res_center) = shape.displacement(0., 0., 0.).unwrap();
        let (target, target_center) = (0.5, Position::new(0., 0., -0.125));
        assert!(
            (result - target).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        assert!(
            (res_center - target_center).len() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            res_center,
            target_center
        );
        let (result, res_center) = shape.displacement(30., -45., 0.1).unwrap();
        let (target, target_center) = (0.9788957917996584, Position::new(-0.01951159263505106, 0.006684388264638706, -0.003025428479409796));
        assert!(
            (result - target).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        assert!(
            (res_center - target_center).len() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            res_center,
            target_center
        );
        test_duration.exit();
    }
    //
    #[test]
    fn shape_waterline_area() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test shape_area";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own("test shape_area");

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (position, objects) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            position.clone(),
            objects.clone(),
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(Point3::new(1., 0., 0.)), 1., 0.0000001, 1000);
        let (result, res_center) = shape.waterline_area(0., 0., 0.).unwrap();
        let (target, target_center) = (2.0, Position::new(0., 0., 0.));
        assert!(
            (result - target).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        assert!(
            (res_center - target_center).len() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            res_center,
            target_center
        );
        let (result, res_center) = shape.waterline_area(30., -45., 0.1).unwrap();
        let (target, target_center) = (0.2357120659516771, Position::new(0.8733361526851109, -0.24667230537003976, 0.10374118731217521));
        assert!(
            (result - target).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        assert!(
            (res_center - target_center).len() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            res_center,
            target_center
        );
        test_duration.exit();
    }
    //
    #[test]
    fn shape_inertia() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        let self_id = "test shape_inertia";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own("test shape_inertia");

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (position, objects) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            position.clone(),
            objects.clone(),
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(Point3::new(1., 0., 0.)), 1., 0.0000001, 1000);
        let result = shape.inertia(0., 0., 0.).unwrap();
        let target = (0.003086434965341909, 0.008008016032056088);
        assert!(
            (result.0 - target.0).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        assert!(
            (result.1 - target.1).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        let result = shape.inertia(30., -45., 0.1).unwrap();
        let target = (0.008288197044037073, 0.0038253444304158436);
        assert!(
            (result.0 - target.0).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        assert!(
            (result.1 - target.1).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        test_duration.exit();
    }
    //
    #[test]
    fn shape_aabb() {
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
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(Point3::new(1., 0., 0.)), 1., 0.0000001, 1000);
        let result = shape.aabb(0.).unwrap();
        let target = (2.0, 1.0);
        assert!(
            (result.0 - target.0).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        assert!(
            (result.1 - target.1).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        test_duration.exit();
    }
}
