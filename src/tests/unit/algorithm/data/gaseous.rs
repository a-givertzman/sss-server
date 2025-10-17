use crate::algorithm::entities::{
    data::loads::{AssignmentType, LoadGaseousData},
    Position,
};

pub fn gaseous() -> Vec<LoadGaseousData> {
    [
        //   name,  mass,    general_category, density, volume,  bound_x1, bound_x2, mass_shift_x,mass_shift_y,mass_shift_z,m_f_s_y,  m_f_s_x,     grain_moment
        (
            "Экипаж и багаж",
            1.8,
            AssignmentType::Stores,
            None::<f64>,
            None::<f64>,
            44.31,
            46.31,
            45.31,
            -4.46,
            7.7,
            None::<f64>,
            None::<f64>,
        ),
        (
            "Провизия",
            1.2,
            AssignmentType::Stores,
            None::<f64>,
            None::<f64>,
            40.21,
            42.21,
            41.21,
            4.6,
            8.15,
            None::<f64>,
            None::<f64>,
        ),
    ]
    .iter()
    .enumerate()
    .map(|(i, v)| LoadGaseousData {
        cargo_id: i,
        cargo_name: v.0.to_owned(),
        space_id: i,
        space_name: v.0.to_owned(),
        assigned_id: i,
        assigment_type: v.2,
        mass: v.1,
        volume: None,
        mass_shift: Some(Position::new(v.7, v.8, v.9)),
    })
    .collect()
    //     LoadLiquidArray::from(
    //        LoadLiquidData{// ID груза // Имя груза // ID помещения // Имя помещения // ID assigned // Тип назначения груза // Тип жидкого груза // масса, т  // Плотность
}
