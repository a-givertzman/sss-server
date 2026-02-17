use crate::data::structs::Ship;

#[allow(dead_code)]
pub(crate) fn ship() -> Ship {
    Ship{
        name: "Belogorodskaya ARK-20231".to_owned(),
        ship_type: "General dry cargo ship".to_owned(),
        navigation_area: "R2".to_owned(),
        p_v: 252., 
        m: 0.52,
        freeboard_type: "B".to_owned(),
    }
}