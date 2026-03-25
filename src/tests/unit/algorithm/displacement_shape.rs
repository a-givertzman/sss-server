#[cfg(test)]

mod tests {
    use crate::algorithm::entities::{Position, model_cached::DisplacementShape};
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use nalgebra::Vector3;
    use sal_core::dbg::Dbg;
    use std::time::Duration;
    use testing::stuff::max_test_duration::TestDuration;
    //
    #[test]
    fn displacement_shape_volume() {
        DebugSession::new()
            .filter(LogLevel::Debug)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
        let self_id = "test displacement_shape_volume";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own(self_id);

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (vertices, indices) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            vertices,
            indices,
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(0.), 1., 0.0000001, 1000);
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
        let (target, target_center) = (0.543301270189222, Position::new(-0.43992243306080187, 0.03834582114280252, -0.016604227619320705));
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
    fn displacement_shape_waterline_area() {
        DebugSession::new()
            .filter(LogLevel::Debug)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
        let self_id = "test displacement_shape_waterline_area";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own(self_id);

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (vertices, indices) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            vertices,
            indices,
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(0.), 1., 0.0000001, 1000);
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
        let (target, target_center) = (0.7071067811993513, Position::new(0.08660254037828123, 0., 0.));
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
    fn displacement_shape_inertia() {
        DebugSession::new()
            .filter(LogLevel::Debug)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
        let self_id = "test displacement_shape_inertia";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own(self_id);

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (vertices, indices) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            vertices,
            indices,
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(0.), 1., 0.0000001, 1000);
        let result = shape.inertia(0., 0., 0.).unwrap();
        let target = (0.17517735803621196, 0.6869528572637689);
        assert!(
            (result.0 - target.0).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result.0,
            target.0
        );
        assert!(
            (result.1 - target.1).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result.1,
            target.1
        );
        let result = shape.inertia(30., -45., 0.1).unwrap();
        let target = (0.05185119345244865, 0.05628635758278952);
        assert!(
            (result.0 - target.0).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result.0,
            target.0
        );
        assert!(
            (result.1 - target.1).abs() < epsilon,
            "\nresult: {:?}\ntarget: {:?}",
            result.1,
            target.1
        );
        test_duration.exit();
    }
    //
    #[test]
    fn displacement_shape_aabb() {
        DebugSession::new()
            .filter(LogLevel::Debug)
            .module("api_tools", LogLevel::Error)
            .module("sal_sync", LogLevel::Error)
            .module("ena", LogLevel::Error)
            .init();
        let self_id = "test displacement_shape_aabb";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        let dbg = Dbg::own(self_id);

        let cuboid = parry3d_f64::shape::Cuboid::new(Vector3::new(1.0, 0.5, 0.25));
        let (vertices, indices) = cuboid.to_trimesh();
        let mesh = parry3d_f64::shape::TriMesh::with_flags(
            vertices,
            indices,
            parry3d_f64::shape::TriMeshFlags::all(),
        )
        .ok();
        let epsilon = 0.0000001;
        let shape = DisplacementShape::new(&dbg, mesh, None, Some(1.), 1., 0.0000001, 1000);
        let result = shape.waterline_size(0.).unwrap();
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
