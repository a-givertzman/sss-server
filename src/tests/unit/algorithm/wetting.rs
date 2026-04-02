#[cfg(test)]
mod tests {
    use crate::algorithm::context::context_access::ContextRead;
    use crate::{
        algorithm::{
            entities::Bounds,
            eval::{WettingCtx, wetting::eval::WettingEval},
        },
        kernel::Eval,
        prelude::{Context, InitialCtx},
        kernel::types::eval_result::EvalResult,
    };
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;

    // Импортируем оригинальный тип штучного груза из вашей кодовой базы
    use crate::algorithm::entities::data::loads::LoadUnitData;
    use crate::algorithm::entities::data::loads::{AssignmentType, UnitCargoType};

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

    fn init_each() {}

    #[test]
    fn wetting() {
        init_once();
        init_each();
        
        let dbg = "wetting";
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
        test_duration.run().unwrap();

        // 1. Создаем разбиение на 3 шпации по 10 метров (всего 30 метров)
        let bounds = Bounds::from_min_max(0.0, 30.0, 3).unwrap();

        // 2. Создаем тестовые штучные грузы
        // Груз 1: лежит в шпациях 1 и 2 (от 5 до 15). Масса 10, намокание 0.5 (50%)
        let unit1 = LoadUnitData {
            cargo_id: 1,
            cargo_name: "Timber_1".to_string(),
            code: "HOLD_1".to_string(),
            space_name: "Hold 1".to_string(),
            assignment_id: 1,
            assigment_type: AssignmentType::CargoLoad, // Подставьте актуальный дефолт
            cargo_type: UnitCargoType::Timber,       // Подставьте актуальный дефолт
            mass: 10.0,
            
            // Оставляем None, чтобы сработал ваш алгоритм вычисления геометрического центра
            mass_shift_x: None, 
            mass_shift_y: None,
            mass_shift_z: None,
            
            stowage_factor: None,
            permeability: Some(0.5),
            volume: None,
            icing_area: None,
            centre_of_icing_area_x: None,
            centre_of_icing_area_y: None,
            centre_of_icing_area_z: None,
            windage_area: None,
            centre_of_windage_area_x: None,
            centre_of_windage_area_y: None,
            centre_of_windage_area_z: None,
            
            // Границы по X: 5.0 .. 15.0. Центр будет на 10.0
            bound_x1: Some(5.0),
            bound_x2: Some(15.0),
            // Границы по Y: -2.0 .. 2.0. Центр будет на 0.0
            bound_y1: Some(-2.0),
            bound_y2: Some(2.0),
            // Границы по Z: 4.0 .. 6.0. Центр будет на 5.0
            bound_z1: Some(4.0),
            bound_z2: Some(6.0),
        };

        // Груз 2: лежит целиком в шпации 3 (от 22 до 26). Масса 20, намокание 0.1 (10%)
        let unit2 = LoadUnitData {
            cargo_id: 2,
            cargo_name: "Timber_2".to_string(),
            code: "HOLD_2".to_string(),
            space_name: "Hold 2".to_string(),
            assignment_id: 2,
            assigment_type: AssignmentType::CargoLoad,
            cargo_type: UnitCargoType::Timber,
            mass: 20.0,
            
            mass_shift_x: None,
            mass_shift_y: None,
            mass_shift_z: None,
            
            stowage_factor: None,
            permeability: Some(0.1),
            volume: None,
            icing_area: None,
            centre_of_icing_area_x: None,
            centre_of_icing_area_y: None,
            centre_of_icing_area_z: None,
            windage_area: None,
            centre_of_windage_area_x: None,
            centre_of_windage_area_y: None,
            centre_of_windage_area_z: None,
            
            // Границы по X: 22.0 .. 26.0. Центр будет на 24.0
            bound_x1: Some(22.0),
            bound_x2: Some(26.0),
            // Границы по Y: -1.0 .. 1.0. Центр будет на 0.0
            bound_y1: Some(-1.0),
            bound_y2: Some(1.0),
            // Границы по Z: 5.0 .. 7.0. Центр будет на 6.0
            bound_z1: Some(5.0),
            bound_z2: Some(7.0),
        };

        // 3. Создаем InitialCtx и наполняем его данными
        let mut initial = InitialCtx::new("1", "NULL", bounds);
        initial.unit = Some(vec![unit1, unit2]); // Прокидываем грузы

        let ctx = Context::new(initial);
        let mock_eval = MockEval { ctx };

        // 4. Выполняем расчет
        let result: WettingCtx = WettingEval::new(dbg, mock_eval)
            .eval(())
            .unwrap()
            .read();

        // РУЧНОЙ РАСЧЕТ ДЛЯ ЭТАЛОНА (TARGET):
        // Масса намокания:
        // Груз 1: 10.0 * 0.5 = 5.0
        // Груз 2: 20.0 * 0.1 = 2.0
        // Итого масса: 5.0 + 2.0 = 7.0
        let target_mass = 7.0;

        // Моменты и Позиция:
        // Момент X: 5.0 * 10.0 (центр 1) + 2.0 * 24.0 (центр 2) = 50.0 + 48.0 = 98.0
        // Момент Z: 5.0 * 5.0 (центр 1) + 2.0 * 6.0 (центр 2) = 25.0 + 12.0 = 37.0
        let target_moment_x = 98.0;
        let target_moment_z = 37.0;

        let target_pos_x = target_moment_x / target_mass; // 14.0
        let target_pos_z = target_moment_z / target_mass; // 5.2857...

        let epsilon = 1e-5;

        // Проверка массы
        assert!(
            (result.mass - target_mass).abs() < epsilon,
            "\n[Mass Mismatch]\nresult: {:?}\ntarget: {:?}",
            result.mass,
            target_mass
        );

        // Проверка координат ЦТ
        let result_pos = result.moment.to_pos(result.mass);
        
        assert!(
            (result_pos.x() - target_pos_x).abs() < epsilon,
            "\n[Position X Mismatch]\nresult: {:?}\ntarget: {:?}",
            result_pos.x(),
            target_pos_x
        );

        assert!(
            (result_pos.z() - target_pos_z).abs() < epsilon,
            "\n[Position Z Mismatch]\nresult: {:?}\ntarget: {:?}",
            result_pos.z(),
            target_pos_z
        );

        // Проверка распределения по шпациям (mass_values):
        // Груз 1 (длина 10м: от 5 до 15): 
        //   - В шпацию 1 (0-10) попадает 5м (половина). Масса = 5.0 * 0.5 = 2.5
        //   - В шпацию 2 (10-20) попадает 5м (половина). Масса = 5.0 * 0.5 = 2.5
        // Груз 2 (длина 4м: от 22 до 26):
        //   - В шпацию 3 (20-30) попадает целиком. Масса = 2.0
        let target_values = vec![2.5, 2.5, 2.0];
        
        assert_eq!(result.mass_values.len(), target_values.len());
        for i in 0..target_values.len() {
            assert!(
                (result.mass_values[i] - target_values[i]).abs() < epsilon,
                "\n[Frame {} Mismatch]\nresult: {:?}\ntarget: {:?}",
                i, result.mass_values[i], target_values[i]
            );
        }

        test_duration.exit();
    }

    /// Вспомогательный MockEval, чтобы прокинуть Context в WettingEval
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
