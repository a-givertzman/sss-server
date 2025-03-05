///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct InitialCtx {
    pub ship_id: usize,
    /// разбиение на шпации - фреймы
    pub bounds: Option<Vec<(f64, f64)>>,

    // /// Тип судна
    // pub ship_type: ShipType,
    // /// Параметры района плавания судна  
    // pub navigation_area: NavigationAreaData,
    // /// Масса льда на квадратный метр площади горизонтальной поверхности
    // /// палубного лесного груза
    // pub icing_m_timber: f64,
    // /// Масса льда на квадратный метр площади парусности
    // /// при учете полного обледенения
    // pub icing_m_v_full: f64,
    // /// Масса льда на квадратный метр площади парусности  
    // /// при учете частичного обледенения
    // pub icing_m_v_half: f64,
    // /// Масса льда на квадратный метр площади горизонтальной
    // /// поверхности при учете полного обледенения
    // pub icing_m_h_full: f64,
    // /// Масса льда на квадратный метр площади горизонтальной  
    // /// поверхности при учете частичного обледенения
    // pub icing_m_h_half: f64,
    // /// Cтепень намокания палубного лесного груза, %
    // pub wetting_timber: f64,
    // /// Безразмерный множитель Х_1 для расчета качки, Табл. 2.1.5.1-1
    // pub multipler_x1: MultiplerX1Array,
    // /// Безразмерный множитель Х_2 для расчета качки, Табл. 2.1.5.1-2
    // pub multipler_x2: MultiplerX2Array,
    // /// Безразмерный множитель S для расчета качки, Табл. 2.1.5.1-3
    // pub multipler_s: MultiplerSArray,
    // /// Коэффициент k для судов, имеющих скуловые кили или
    // /// брусковый киль для расчета качки, Табл. 2.1.5.2
    // pub coefficient_k: CoefficientKArray,
    // /// Коэффициент k_theta учитывающий особенности качки судов смешанного типа
    // pub coefficient_k_theta: CoefficientKThetaArray,
    // /// Длинна корпуса судна между перпендикулярами
    // pub length_lbp: f64,
    // /// Длинна корпуса судна полная
    // pub length_loa: f64,
    // /// Ширина корпуса судна
    // pub width: f64,
    // /// Тип надводного борта
    // pub freeboard_type: String,
    // /// Суммарая площадь проекции носа судна на диаметральную плоскость
    // pub bow_area_min: f64,
    // /// Отстояние миделя от нулевого шпангоута
    // pub midship: f64,
    // /// Overall height up to non-removable parts
    // pub overall_height: f64,
    // /// Calculated minimum bow height
    // pub bow_h_min: f64,
    // /// Minimum allowable trim
    // pub aft_trim: f64,
    // /// Maximum allowable trim
    // pub forward_trim: f64,        
    // /// Эксплуатационная скорость судна, m/s
    // pub velocity: f64,
    // /// Дедвейт
    // pub deadweight: f64,
    // /// Cуммарная габаритная площадь скуловых килей,
    // /// либо площадь боковой проекции брускового киля
    // pub keel_area: Option<f64>,
    // /// плотность воды
    // pub water_density: f64,
    // /// отстояние центра тяжести постоянной массы судна по x  
    // pub const_mass_shift_x: f64,
    // /// отстояние центра тяжести постоянной массы судна по y
    // pub const_mass_shift_y: f64,
    // /// отстояние центра тяжести постоянной массы судна по z
    // pub const_mass_shift_z: f64,
    // /// Минимальная осадка, м
    // pub draught_min: f64,
    // /// Высота борта, м
    // pub moulded_depth: f64,
    // /// Коэффициент увеличения площади парусности несплощной
    // /// поверхности при учете обледенения
    // pub icing_coef_v_area_full: f64,
    // pub icing_coef_v_area_half: f64,
    // pub icing_coef_v_area_zero: f64,
    // /// Коэффициент увеличения статического момента
    // /// площади парусности несплощной поверхности
    // /// при учете обледенения
    // pub icing_coef_v_moment_full: f64,
    // pub icing_coef_v_moment_half: f64,
    // pub icing_coef_v_moment_zero: f64,
    // /// Кривая отстояния центра тяжести ватерлинии по длине от миделя  
    // pub center_waterline: Vec<(f64, Vec<(f64, f64)>)>,
    // /// Длинна корпуса судна по ватерлинии
    // pub waterline_length: Vec<(f64, f64)>,
    // /// Ширина корпуса судна по ватерлинии
    // pub waterline_breadth: Vec<(f64, Vec<(f64, f64)>)>,
    // /// Площадь ватерлинии
    // pub waterline_area: Vec<(f64, Vec<(f64, f64)>)>,
    // /// Отстояние по вертикали центра площади проекции подводной части корпуса
    // pub volume_shift: Vec<(f64, f64)>,
    // /// кривая продольного метацентрического радиуса
    // pub rad_long: Vec<(f64, Vec<(f64, f64)>)>,
    // /// кривая поперечного метацентрического радиуса
    // pub rad_trans: Vec<(f64, Vec<(f64, f64)>)>,
    // /// Минимальная допустимая метацентрическая высота деления на отсеки
    // pub h_subdivision: Vec<(f64, f64)>,
    // /// кривая средней осадки
    // pub mean_draught: Vec<(f64, Vec<(f64, f64)>)>,
    // /// Кривые плечей остойчивости формы
    // pub pantocaren: PantocarenVec,
    // /// Угол заливания отверстий
    // pub flooding_angle: Vec<(f64, Vec<(f64, f64)>)>,
    // /// Угол входа верхней палубы в воду
    // pub entry_angle: Vec<(f64, Vec<(f64, f64)>)>,
    // /// Координаты отметок заглубления на корпусе судна
    // pub draft_mark: Vec<DraftMarkParsedData>,
    // /// Координаты отметок осадок на корпусе судна
    // pub load_line: Vec<LoadLineParsedData>,
    // /// Координаты и диаметр винтов судна
    // pub screw: Vec<ScrewParsedData>,
    // /// Нагрузка судна без жидких грузов   
    // pub cargoes: Vec<LoadCargo>,
    // /// Нагрузка судна: цистерны и трюмы   
    // pub compartments: Vec<CompartmentData>,
    // /// Постоянная нагрузка на судно
    // pub load_constants: Vec<LoadConstantData>,
    // /// Площадь горизонтальных поверхностей для остойчивости
    // pub area_h_stab: Vec<HStabArea>,
    // /// Площадь и моменты поверхности парусности для остойчивости
    // pub area_v_stab: stability::VerticalAreaArray,
    // /// Площадь поверхности парусности для прочности
    // pub area_v_str: Vec<strength::VerticalArea>,
    // /// Cуммарая площадь проекции на диаметральную плоскость от осадки, м^2
    // pub bow_area: Vec<(f64, f64)>,
}
impl InitialCtx {
    ///
    /// Struct constructor
    /// - 'ship_id' - the identifier of the ship in the database
    pub fn new(ship_id: usize) -> Self {
        Self {
            ship_id,
            bounds: None,
        }
    }
}
//
//
impl Default for InitialCtx {
    ///
    /// Struct constructor
    /// - 'storage_initial_data' - [Storage] instance, where store initial data
    fn default() -> Self {
        Self {
            ship_id: 0,
            bounds: None,
        }
    }
}
