use crate::data::structs::Voyage;

#[allow(dead_code)]
pub(crate) fn voyage() -> Voyage {
    Voyage{
        density: 1.025,
        operational_speed: 16.,
        wetting_timber: 10.,
        icing_type: "none".to_owned(),
        icing_timber_type: "full".to_owned(),
    }
}