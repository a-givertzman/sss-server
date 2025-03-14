//! Структуры для преобразования данных из формата данных DB
//! в формат пригодный для создания объектов.
use std::rc::Rc;

use crate::kernel::error::error::Error;

use super::{icing_stab::IcingStabType, icing_timber::IcingTimberType, math::Position, stability::{multipler_s::MultiplerSArray, ship_type::ShipType, *}, strength::ParsedFrameData, *};

/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug)]
pub struct ParsedShipData {
    /// Тип судна
    pub ship_type: ShipType,
    /// Параметры района плавания судна  
    pub navigation_area: NavigationAreaData,
    /// Тип обледенения
    pub icing_stab: IcingStabType,
    /// Тип обледенения палубного груза - леса
    pub icing_timber_stab: IcingTimberType,
    /// Масса льда на квадратный метр площади горизонтальной поверхности
    /// палубного лесного груза
    pub icing_m_timber: f64,
    /// Масса льда на квадратный метр площади парусности
    /// при учете полного обледенения
    pub icing_m_v_full: f64,
    /// Масса льда на квадратный метр площади парусности  
    /// при учете частичного обледенения
    pub icing_m_v_half: f64,
    /// Масса льда на квадратный метр площади горизонтальной
    /// поверхности при учете полного обледенения
    pub icing_m_h_full: f64,
    /// Масса льда на квадратный метр площади горизонтальной  
    /// поверхности при учете частичного обледенения
    pub icing_m_h_half: f64,
    /// Cтепень намокания палубного лесного груза, %
    pub wetting_timber: f64,
    /// Безразмерный множитель Х_1 для расчета качки, Табл. 2.1.5.1-1
    pub multipler_x1: MultiplerX1Array,
    /// Безразмерный множитель Х_2 для расчета качки, Табл. 2.1.5.1-2
    pub multipler_x2: MultiplerX2Array,
    /// Безразмерный множитель S для расчета качки, Табл. 2.1.5.1-3
    pub multipler_s: MultiplerSArray,
    /// Коэффициент k для судов, имеющих скуловые кили или
    /// брусковый киль для расчета качки, Табл. 2.1.5.2
    pub coefficient_k: CoefficientKArray,
    /// Коэффициент k_theta учитывающий особенности качки судов смешанного типа
    pub coefficient_k_theta: CoefficientKThetaArray,
    /// Длинна корпуса судна между перпендикулярами
    pub length_lbp: f64,
    /// Длинна корпуса судна полная
    pub length_loa: f64,
    /// Ширина корпуса судна
    pub width: f64,
    /// Тип надводного борта
    pub freeboard_type: String,
    /// Суммарая площадь проекции носа судна на диаметральную плоскость
    pub bow_area_min: f64,
    /// Отстояние миделя от нулевого шпангоута
    pub midship: f64,
    /// Overall height up to non-removable parts
    pub overall_height: f64,
    /// Calculated minimum bow height
    pub bow_h_min: f64,
    /// Minimum allowable trim
    pub aft_trim: f64,
    /// Maximum allowable trim
    pub forward_trim: f64,        
    /// Эксплуатационная скорость судна, m/s
    pub velocity: f64,
    /// Дедвейт
    pub deadweight: f64,
    /// Cуммарная габаритная площадь скуловых килей,
    /// либо площадь боковой проекции брускового киля
    pub keel_area: Option<f64>,
    /// разбиение на шпации - фреймы
    pub bounds: Vec<(f64, f64)>,
    /// плотность воды
    pub water_density: f64,
    /// отстояние центра тяжести постоянной массы судна по x  
    pub const_mass_shift_x: f64,
    /// отстояние центра тяжести постоянной массы судна по y
    pub const_mass_shift_y: f64,
    /// отстояние центра тяжести постоянной массы судна по z
    pub const_mass_shift_z: f64,
    /// Минимальная осадка, м
    pub draught_min: f64,
    /// Высота борта, м
    pub moulded_depth: f64,
    /// Коэффициент увеличения площади парусности несплощной
    /// поверхности при учете обледенения
    pub icing_coef_v_area_full: f64,
    pub icing_coef_v_area_half: f64,
    pub icing_coef_v_area_zero: f64,
    /// Коэффициент увеличения статического момента
    /// площади парусности несплощной поверхности
    /// при учете обледенения
    pub icing_coef_v_moment_full: f64,
    pub icing_coef_v_moment_half: f64,
    pub icing_coef_v_moment_zero: f64,
    /// Кривая отстояния центра тяжести ватерлинии по длине от миделя  
    pub center_waterline: Vec<(f64, Vec<(f64, f64)>)>,
    /// Длинна корпуса судна по ватерлинии
    pub waterline_length: Vec<(f64, f64)>,
    /// Ширина корпуса судна по ватерлинии
    pub waterline_breadth: Vec<(f64, Vec<(f64, f64)>)>,
    /// Площадь ватерлинии
    pub waterline_area: Vec<(f64, Vec<(f64, f64)>)>,
    /// Отстояние по вертикали центра площади проекции подводной части корпуса
    pub volume_shift: Vec<(f64, f64)>,
    /// кривая продольного метацентрического радиуса
    pub rad_long: Vec<(f64, Vec<(f64, f64)>)>,
    /// кривая поперечного метацентрического радиуса
    pub rad_trans: Vec<(f64, Vec<(f64, f64)>)>,
    /// Минимальная допустимая метацентрическая высота деления на отсеки
    pub h_subdivision: Vec<(f64, f64)>,
    /// кривая средней осадки
    pub mean_draught: Vec<(f64, Vec<(f64, f64)>)>,
    /// кривая отстояния центра величины погруженной части судна
    pub center_draught_shift: Vec<(f64, Vec<(f64, Position)>)>,
    /// Кривые плечей остойчивости формы
    pub pantocaren: PantocarenVec,
    /// Угол заливания отверстий
    pub flooding_angle: Vec<(f64, Vec<(f64, f64)>)>,
    /// Угол входа верхней палубы в воду
    pub entry_angle: Vec<(f64, Vec<(f64, f64)>)>,
    /// Погруженная площадь шпангоута
    pub frame_area: Vec<ParsedFrameData>,
    /// Координаты отметок заглубления на корпусе судна
    pub draft_mark: Vec<DraftMarkParsedData>,
    /// Координаты отметок осадок на корпусе судна
    pub load_line: Vec<LoadLineParsedData>,
    /// Координаты и диаметр винтов судна
    pub screw: Vec<ScrewParsedData>,
    /// Высота борта на носовом перпендикуляре
    pub bow_board: Vec<BowBoardParsedData>,
    /// Нагрузка судна без жидких грузов   
    pub cargoes: Vec<LoadCargo>,
    /// Нагрузка судна: цистерны и трюмы   
    pub compartments: Vec<CompartmentData>,
    /// Постоянная нагрузка на судно
    pub load_constants: Vec<LoadConstantData>,
    /// Площадь горизонтальных поверхностей для остойчивости
    pub area_h_stab: Vec<HStabArea>,
    // /// Площадь горизонтальных поверхностей для прочности
    // pub area_h_str: Vec<HStrArea>,
    /// Площадь и моменты поверхности парусности для остойчивости
    pub area_v_stab: stability::VerticalAreaArray,
    /// Площадь поверхности парусности для прочности
    pub area_v_str: Vec<strength::VerticalArea>,
    /// Cуммарая площадь проекции на диаметральную плоскость от осадки, м^2
    pub bow_area: Vec<(f64, f64)>,
}
//
impl ParsedShipData {
    /// Парсинг данных в общую структуру. Включает в себя  
    /// проверку данных на корректность.
    #[allow(clippy::too_many_arguments)]
    pub fn parse(
        multipler_x1: MultiplerX1Array,
        multipler_x2: MultiplerX2Array,
        multipler_s: MultiplerSArray,
        coefficient_k: CoefficientKArray,
        coefficient_k_theta: CoefficientKThetaArray,
        icing: IcingArray,
        ship_id: usize,        
        ship: Ship,
        voyage: Voyage,
        ship_parameters: ShipParametersArray,
        // bounds: ComputedFrameDataArray,
        center_waterline: CenterWaterlineArray,
        waterline_length: WaterlineLengthArray,
        waterline_breadth: WaterlineBreadthArray,
        waterline_area: WaterlineAreaArray,
        volume_shift: VolumeShiftArray,
        rad_long: RadLongDataArray,
        rad_trans: RadTransDataArray,
        h_subdivision: MetacentricHeightSubdivisionArray,
        mean_draught: MeanDraughtDataArray,
        center_draught_shift: CenterDraughtShiftArray,
        pantocaren: PantocarenDataArray,
        flooding_angle: FloodingAngleDataArray,
        entry_angle: EntryAngleDataArray,
        bonjean_frame: FrameIndexDataArray,
        frame_area: FrameAreaDataArray,
        draft_mark: DraftMarkDataArray,
        load_line: LoadLineDataArray,
        screw: ScrewDataArray,
        bow_board: BowBoardDataArray,
        cargo: LoadCargoArray,
        containers: ContainerArray,
        bulkhead: BulkheadArray,
        compartments: CompartmentArray,
        hold_parts: CompartmentArray,
        load_constants: LoadConstantArray,
        area_h_stab: HStabAreaArray,
        area_h_str: HStrAreaArray,
        area_v_stab: stability::VerticalAreaArray,
        area_v_str: strength::VerticalAreaArray,
        bow_area: BowAreaDataArray,
    ) -> Result<Rc<Self>, Error> {
        log::info!("result parse begin");
        let ship_data = ship_parameters.data();
        let bonjean_frame = bonjean_frame.data();        
        let frame_area = frame_area.data();
        let mut parsed_frame_area = Vec::new();        
        for (index, x) in bonjean_frame {
            parsed_frame_area.push(ParsedFrameData {
                x,
                immersion_area: frame_area.get(&index).ok_or(format!(
                    "ParsedShipData parse error: no immersion_area for frame index:{}",
                    index
                ))?.to_vec(),
            });
        }
        parsed_frame_area.sort_by(|a, b| a.x.partial_cmp(&b.x).expect("result parsed_frame_area cpm error!"));
        let icing = icing.data();
        let ship_type = ShipType::from_str(&ship.ship_type)?;
        let navigation_area = ship.navigation_area()?;
        let length_lbp = *ship_data.get("LBP").ok_or(format!(
            "ParsedShipData parse error: no length for ship id:{}",
            ship_id
        ))?;
        let length_loa = *ship_data.get("L.O.A").ok_or(format!(
            "ParsedShipData parse error: no length_loa for ship id:{}",
            ship_id
        ))?;
        let width = *ship_data.get("MouldedBreadth").ok_or(format!(
            "ParsedShipData parse error: no width for ship id:{}",
            ship_id
        ))?;
        let freeboard_type = ship.freeboard_type.clone();
        let bow_area_min = *ship_data.get("Calculated minimum bow area").ok_or(format!(
            "ParsedShipData parse error: no bow_area_min for ship id:{}",
            ship_id
        ))?;
        let midship = *ship_data.get("X midship from Fr0").ok_or(format!(
            "ParsedShipData parse error: no midship for ship id:{}",
            ship_id
        ))?;
        let overall_height = *ship_data.get("Overall height up to non-removable parts").ok_or(format!(
            "ParsedShipData parse error: no overall_height for ship id:{}",
            ship_id
        ))?;
        let bow_h_min = *ship_data.get("Calculated minimum bow height").ok_or(format!(
            "ParsedShipData parse error: no bow_h_min for ship id:{}",
            ship_id
        ))?;
        let aft_trim = *ship_data.get("Maximum aft trim").ok_or(format!(
            "ParsedShipData parse error: no aft_trim for ship id:{}",
            ship_id
        ))?;
        let forward_trim = *ship_data.get("Maximum forward trim").ok_or(format!(
            "ParsedShipData parse error: no forward_trim for ship id:{}",
            ship_id
        ))?;
        let velocity = voyage.operational_speed*0.514444444; // knot to m/s
        let deadweight = *ship_data.get("DWT").ok_or(format!(
            "ParsedShipData parse error: no deadweight for ship id:{}",
            ship_id
        ))?;
        let keel_area = ship_data.get("Keel area").copied();
        let water_density = voyage.density;
        let const_mass_shift_x = *ship_data.get("LCG from middle").ok_or(format!(
            "ParsedShipData parse error: no const_mass_shift_x for ship id:{}",
            ship_id
        ))?;
        let const_mass_shift_y = *ship_data.get("TCG from CL").ok_or(format!(
            "ParsedShipData parse error: no const_mass_shift_y for ship id:{}",
            ship_id
        ))?;
        let const_mass_shift_z = *ship_data.get("VCG from BL").ok_or(format!(
            "ParsedShipData parse error: no const_mass_shift_z for ship id:{}",
            ship_id
        ))?;
        let draught_min = *ship_data.get("Minimum draft").ok_or(format!(
            "ParsedShipData parse error: no draught_min for ship id:{}",
            ship_id
        ))?;
        let moulded_depth = *ship_data.get("Moulded depth").ok_or(format!(
            "ParsedShipData parse error: no moulded_depth for ship id:{}",
            ship_id
        ))?;
        let icing_stab = IcingStabType::from_str(&voyage.icing_type)?;
        let icing_timber_stab = IcingTimberType::from_str(&voyage.icing_timber_type)?;
        let icing_m_timber = *icing.get("icing_m_timber").ok_or(format!(
            "ParsedShipData parse error: no icing_m_timber for ship id:{}",
            ship_id
        ))?;
        let icing_m_v_full = *icing.get("icing_m_v_full").ok_or(format!(
            "ParsedShipData parse error: no icing_m_v_full for ship id:{}",
            ship_id
        ))?;
        let icing_m_v_half = *icing.get("icing_m_v_half").ok_or(format!(
            "ParsedShipData parse error: no icing_m_v_half for ship id:{}",
            ship_id
        ))?;
        let icing_m_h_full = *icing.get("icing_m_h_full").ok_or(format!(
            "ParsedShipData parse error: no icing_m_h_full for ship id:{}",
            ship_id
        ))?;
        let icing_m_h_half = *icing.get("icing_m_h_half").ok_or(format!(
            "ParsedShipData parse error: no icing_m_h_half for ship id:{}",
            ship_id
        ))?;
        let wetting_timber = voyage.wetting_timber*0.01;
        let icing_coef_v_area_full = *icing.get("icing_coef_v_area_full").ok_or(format!(
            "ParsedShipData parse error: no icing_coef_v_area_full for ship id:{}",
            ship_id
        ))?;
        let icing_coef_v_area_half = *icing.get("icing_coef_v_area_half").ok_or(format!(
            "ParsedShipData parse error: no icing_coef_v_area_half for ship id:{}",
            ship_id
        ))?;
        let icing_coef_v_area_zero = *icing.get("icing_coef_v_area_zero").ok_or(format!(
            "ParsedShipData parse error: no icing_coef_v_area_zero for ship id:{}",
            ship_id
        ))?;
        let icing_coef_v_moment_full = *icing.get("icing_coef_v_moment_full").ok_or(format!(
            "ParsedShipData parse error: no icing_coef_v_moment_full for ship id:{}",
            ship_id
        ))?;
        let icing_coef_v_moment_half = *icing.get("icing_coef_v_moment_half").ok_or(format!(
            "ParsedShipData parse error: no icing_coef_v_moment_half for ship id:{}",
            ship_id
        ))?;
        let icing_coef_v_moment_zero = *icing.get("icing_coef_v_moment_zero").ok_or(format!(
            "ParsedShipData parse error: no icing_coef_v_moment_zero for ship id:{}",
            ship_id
        ))?;
        let mut cargoes = cargo.data();
        cargoes.append(&mut containers.data());   
        cargoes.append(&mut bulkhead.data());        
        let mut compartments = compartments.data();
        compartments.append(&mut hold_parts.data());
        log::info!("result parse ok");

      /*  let result = Box::new( Self {
            ship_type,
            navigation_area,
            multipler_x1,
            multipler_x2,
            multipler_s,
            coefficient_k,
            coefficient_k_theta,
            length_lbp,
            length_loa,
            width,
            freeboard_type, 
            bow_area_min,
            midship,
            overall_height,
            bow_h_min,
            aft_trim,
            forward_trim,
            velocity,
            deadweight,
            keel_area,
            water_density,
            const_mass_shift_x,
            const_mass_shift_y,
            const_mass_shift_z,
            draught_min,
            moulded_depth,
            icing_stab,
            icing_timber_stab,
            icing_m_timber,
            icing_m_v_full,
            icing_m_v_half,
            icing_m_h_full,
            icing_m_h_half,
            wetting_timber,
            icing_coef_v_area_full,
            icing_coef_v_area_half,
            icing_coef_v_area_zero,
            icing_coef_v_moment_full,
            icing_coef_v_moment_half,
            icing_coef_v_moment_zero,
            bounds: bounds.data(),
            center_waterline: center_waterline.data(),
            waterline_length: waterline_length.data(),
            waterline_breadth: waterline_breadth.data(),
            waterline_area: waterline_area.data(),
            volume_shift: volume_shift.data(),
            rad_long: rad_long.data(),
            rad_trans: rad_trans.data(),
            h_subdivision: h_subdivision.data(),
            mean_draught: mean_draught.data(),
            center_draught_shift: center_draught_shift.data(),
            pantocaren: pantocaren.data(),
            flooding_angle: flooding_angle.data(),
            entry_angle: entry_angle.data(),
            frame_area: parsed_frame_area,
            draft_mark: draft_mark.draft_data(),
            load_line: load_line.load_line_data(),
            screw: screw.data(),
            bow_board: bow_board.bow_board_data(),
            cargoes,
            compartments,
            load_constants: load_constants.data(),
            area_h_stab: area_h_stab.data(),
            area_h_str: area_h_str.data(),
            area_v_stab,
            area_v_str: area_v_str.data(),
            bow_area: bow_area.data(),
        }
        .check());
*/
        match ( Self {
            ship_type,
            navigation_area,
            multipler_x1,
            multipler_x2,
            multipler_s,
            coefficient_k,
            coefficient_k_theta,
            length_lbp,
            length_loa,
            width,
            freeboard_type, 
            bow_area_min,
            midship,
            overall_height,
            bow_h_min,
            aft_trim,
            forward_trim,
            velocity,
            deadweight,
            keel_area,
            water_density,
            const_mass_shift_x,
            const_mass_shift_y,
            const_mass_shift_z,
            draught_min,
            moulded_depth,
            icing_stab,
            icing_timber_stab,
            icing_m_timber,
            icing_m_v_full,
            icing_m_v_half,
            icing_m_h_full,
            icing_m_h_half,
            wetting_timber,
            icing_coef_v_area_full,
            icing_coef_v_area_half,
            icing_coef_v_area_zero,
            icing_coef_v_moment_full,
            icing_coef_v_moment_half,
            icing_coef_v_moment_zero,
            bounds: bounds.data(),
            center_waterline: center_waterline.data(),
            waterline_length: waterline_length.data(),
            waterline_breadth: waterline_breadth.data(),
            waterline_area: waterline_area.data(),
            volume_shift: volume_shift.data(),
            rad_long: rad_long.data(),
            rad_trans: rad_trans.data(),
            h_subdivision: h_subdivision.data(),
            mean_draught: mean_draught.data(),
            center_draught_shift: center_draught_shift.data(),
            pantocaren: pantocaren.data(),
            flooding_angle: flooding_angle.data(),
            entry_angle: entry_angle.data(),
            frame_area: parsed_frame_area,
            draft_mark: draft_mark.draft_data(),
            load_line: load_line.load_line_data(),
            screw: screw.data(),
            bow_board: bow_board.bow_board_data(),
            cargoes,
            compartments,
            load_constants: load_constants.data(),
            area_h_stab: area_h_stab.data(),
            area_h_str: area_h_str.data(),
            area_v_stab,
            area_v_str: area_v_str.data(),
            bow_area: bow_area.data(),
        }.check()) {
            Ok(v) => {
                log::info!("result check ok");
                Ok(Rc::new(v))
            },
            Err(e) => {
                log::error!("result check error: {}", e);
                Err(e)
            }
        }
    }
}
