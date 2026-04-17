#[cfg(test)]
mod tests {
    use crate::algorithm::context::context_access::ContextRead;
    use crate::algorithm::eval::strength::area::eval::AreaStrEval;
    use crate::prelude::ContextWrite;
    use crate::{
        algorithm::{
            entities::{Bound, Bounds},
            eval::strength::AreaStrCtx, // Используем правильный контекст площадей
        },
        kernel::Eval,
        prelude::{Context, InitialCtx},
        kernel::types::{Arc, RwLock, eval_result::EvalResult},
    };
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;

    // Импортируем оригинальные типы из вашей кодовой базы
    use crate::algorithm::entities::data::loads::{LoadUnitData, AssignmentType, UnitCargoType};
    use crate::algorithm::eval::icing_timber_bound::ctx::{IcingTimberBoundCtx, IcingTimberType};
    use crate::algorithm::entities::ship_model::ship_model::ShipModel;

    static INIT: Once = Once::new();

    fn init_once() {
        INIT.call_once(|| {
            DebugSession::new()
                .filter(LogLevel::Debug)
                .module("api_tools", LogLevel::Error)
                .module("sal_sync", LogLevel::Error)
                .module("ena", LogLevel::Error)
                .init();
        })
    }

    #[test]
    fn strength_area() {
        init_once();
        
        let self_id = "test strength_area";
        println!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();

        // 1. Создаем разбиение на 3 шпации по 10 метров (всего 30 метров)
        let bounds = Bounds::from_min_max(0.0, 30.0, 3).unwrap();

        // 2. Создаем тестовый лесной груз (влияет на парусность и вычитается из площади палуб)
        // Груз лежит в шпациях 1 и 2 (от 5 до 15). Высота от 4 до 6.
        let timber = LoadUnitData {
            cargo_id: 1,
            cargo_name: "Timber".to_string(),
            code: "DECK".to_string(),
            space_name: "Deck".to_string(),
            assignment_id: 1,
            assigment_type: AssignmentType::CargoLoad,
            cargo_type: UnitCargoType::Timber, // ВАЖНО: тип "Лес"
            mass: 10.0,
            mass_shift_x: None, mass_shift_y: None, mass_shift_z: None,
            stowage_factor: None,
            permeability: None,
            volume: None,
            
            icing_area: Some(40.0), // Площадь обледенения (горизонтальная)
            centre_of_icing_area_x: None, centre_of_icing_area_y: None, centre_of_icing_area_z: None,
            
            windage_area: Some(20.0), // Площадь парусности
            centre_of_windage_area_x: None, centre_of_windage_area_y: None, centre_of_windage_area_z: None,
            
            bound_x1: Some(5.0), bound_x2: Some(15.0),
            bound_y1: Some(-2.0), bound_y2: Some(2.0),
            bound_z1: Some(4.0), bound_z2: Some(6.0),
        };

         // 3. Создаем InitialCtx и наполняем его
        let mut initial = InitialCtx::new("1", "NULL", bounds);
        initial.unit = Some(vec![timber]);

        let ctx = Context::new(initial);

        // ИСПРАВЛЕНО: Создаем IcingTimberBoundCtx строго через ваш оригинальный конструктор
        // Передаем ширину 10.0, длину 30.0 и тип обледенения Full
        let timber_bound = IcingTimberBoundCtx::new(
            10.0, 
            30.0, 
            IcingTimberType::Full
        );

        let mock_eval = MockEval { ctx: ctx.write(timber_bound).unwrap() };

        // 4. Мокаем ShipModel
        // В вашем проекте ShipModel возвращает структуру с полями .v и .h для strength_area()
        // Предположим, что мы создали простой мок, возвращающий базовые эпюры парусности (по 10м2 на шпацию)
        // и базовые эпюры горизонтальных площадей (по 50м2 на шпацию)
        let ship = ShipModel::create_test_fake_strength(
            vec![10.0, 10.0, 10.0], // Базовая парусность корпуса по шпациям
            vec![50.0, 50.0, 50.0], // Базовая горизонтальная площадь по шпациям
        );
        let model = Arc::new(RwLock::new(ship));

        // 5. Выполняем расчет
        let dbg = sal_core::dbg::Dbg::new("test", "strength_area");
        let result: AreaStrCtx = AreaStrEval::new(dbg, model, mock_eval)
            .eval(())
            .unwrap()
            .read();

        // РУЧНОЙ РАСЧЕТ ДЛЯ ЭТАЛОНА (TARGET):
        //
        // Шпация 1 (0-10 м):
        // - Парусность: корпус (10.0) + груз (длина пересечения 5м * высота груза (6-4=2м)) = 20.0
        // - Лес горизонт: попадает 5м из 10м длины груза = 40.0 * (5/10) = 20.0
        // - Палуба горизонт: корпус (50.0) - полный лес в этой шпации (20.0) = 30.0
        //
        // Шпация 2 (10-20 м):
        // - Парусность: корпус (10.0) + груз (длина пересечения 5м * высота груза (2м)) = 20.0
        // - Лес горизонт: попадает 5м из 10м длины груза = 20.0
        // - Палуба горизонт: корпус (50.0) - полный лес в этой шпации (20.0) = 30.0
        //
        // Шпация 3 (20-30 м):
        // - Парусность: только корпус = 10.0
        // - Лес горизонт: 0.0
        // - Палуба горизонт: только корпус = 50.0

        let target_area_v = vec![20.0, 20.0, 10.0];
        let target_area_h = vec![30.0, 30.0, 50.0];
        let target_timber_h = vec![20.0, 20.0, 0.0];

        let epsilon = 1e-5;

        // Проверяем распределение парусности
        for i in 0..target_area_v.len() {
            assert!(
                (result.area_v[i] - target_area_v[i]).abs() < epsilon,
                "\n[Area V Mismatch at frame {}]\nresult: {:?}\ntarget: {:?}",
                i, result.area_v[i], target_area_v[i]
            );
        }

        // Проверяем распределение горизонтальных площадей корпуса
        for i in 0..target_area_h.len() {
            assert!(
                (result.area_h[i] - target_area_h[i]).abs() < epsilon,
                "\n[Area H Mismatch at frame {}]\nresult: {:?}\ntarget: {:?}",
                i, result.area_h[i], target_area_h[i]
            );
        }

        // Проверяем распределение площадей леса
        for i in 0..target_timber_h.len() {
            assert!(
                (result.timber_icing_h[i] - target_timber_h[i]).abs() < epsilon,
                "\n[Timber Icing H Mismatch at frame {}]\nresult: {:?}\ntarget: {:?}",
                i, result.timber_icing_h[i], target_timber_h[i]
            );
        }

        test_duration.exit();
    }

    /// Вспомогательный MockEval, чтобы прокинуть Context в AreaStrEval
    #[derive(Debug, Clone)]
    struct MockEval {
        pub ctx: Context,
    }

    impl Eval<(), EvalResult> for MockEval {
        fn eval(&self, _: ()) -> EvalResult {
            Result::Ok(self.ctx.clone())
        }
    }
}
