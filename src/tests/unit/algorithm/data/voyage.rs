use crate::algorithm::entities::data::Voyage;

#[allow(dead_code)]
pub(crate) fn voyage() -> Voyage {
    Voyage{
        density: 1.025,
        operational_speed: 16.,
        icing_type: "none".to_owned(),
        icing_timber_type: "full".to_owned(),
        area: Some("sea".to_owned()),
        course_angle: 120.,
        wave_heading_angle: 90.,
        wave_length: 10.,
        current_speed: 10.,
    }
}