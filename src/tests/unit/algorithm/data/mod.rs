pub(crate) mod area_h_str;
pub(crate) mod area_v_str;
pub(crate) mod ship;
pub(crate) mod ship_parameters;
pub(crate) mod voyage;
pub(crate) mod icing;
pub(crate) mod load_constant;

pub(crate) use area_h_str::area_h_str;
pub(crate) use area_v_str::area_v_str;
pub(crate) use ship::ship;
pub(crate) use ship_parameters::ship_parameters;
pub(crate) use voyage::voyage;
pub(crate) use icing::icing;
pub(crate) use load_constant::load_constant;
// Bounds::from_min_max(-3.6, 135.5, 200)
/*
#[allow(dead_code)]
pub(crate) fn input_sofia_1() -> Rc<ParsedShipData> {
    ParsedShipData::parse()
}*/