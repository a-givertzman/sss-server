//! Структура с данными для имитации работы модели.  
//! Инициализируется данными для 'Belogorodskaya ARK-20231'
//! 66 вариант расчета Судно порожнем без балласта. Отход (море)

pub(crate) mod area_h_str;
pub(crate) mod area_v_str;
/*
mod area_h_stab;
mod area_v_stab;
mod bonjean_frame;
mod bounds;
mod bow_area;
mod bow_board;
mod bulkhead;
mod center_draught_shift;
mod center_waterline;
mod coefficient_k;
mod coefficient_k_theta;
mod compartment;
mod container;
mod data;
mod draft_mark;
mod entry_angle;
mod flooding_angle;
mod frame_area;
mod h_subdivision;
mod hold_compartment;
mod icing;
mod load_constants;
mod load_line;
mod mean_draught;
mod multipler_s;
mod multipler_x1;
mod multipler_x2;
mod pantocaren;
mod rad_long;
mod rad_trans;
mod screw;
mod ship;
mod voyage;
mod ship_parameters;
mod volume_shift;
mod waterline_area;
mod waterline_breadth;
mod waterline_length;
*/


/*
use std::{collections::HashMap, rc::Rc};

use crate::data::structs::{
    loads::{ContainerArray, LoadCargoArray}, ParsedShipData
};

#[allow(dead_code)]
pub(crate) fn input_data_66() -> Rc<ParsedShipData> {
    ParsedShipData::parse(
        multipler_x1::multipler_x1(),
        multipler_x2::multipler_x2(),
        multipler_s::multipler_s(),
        coefficient_k::coefficient_k(),
        coefficient_k_theta::coefficient_k_theta(),
        icing::icing(),
        1,
        ship::ship(),
        voyage::voyage(),
        ship_parameters::ship_parameters(),
        bounds::bounds(119.95, 59.194, 20),
        center_waterline::center_waterline(),
        waterline_length::waterline_length(),
        waterline_breadth::waterline_breadth(),
        waterline_area::waterline_area(),
        volume_shift::volume_shift(),
        rad_long::rad_long(),
        rad_trans::rad_trans(),
        h_subdivision::h_subdivision(),
        mean_draught::mean_draught(),
        center_draught_shift::center_draught_shift(),
        pantocaren::pantocaren(),
        flooding_angle::flooding_angle(),
        entry_angle::entry_angle(),
        bonjean_frame::bonjean_frame(),
        frame_area::frame_area(),
        draft_mark::draft_mark(),
        load_line::load_line(),
        screw::screw(),
        bow_board::bow_board(),
        LoadCargoArray {
            data: Vec::new(),
            error: HashMap::new(),
        },
        ContainerArray {
            data: Vec::new(),
            error: HashMap::new(),
        },
        bulkhead::bulkhead_27_28(),
        compartment::compartment_100_sea(),
        hold_compartment::hold_compartment_empty(),
        load_constants::load_constants(),
        area_h_stab::area_h_stab(),
        area_h_str::area_h_str(),
        area_v_stab::area_v_stab(),
        area_v_str::area_v_str(),
        bow_area::bow_area(),
    )
    .unwrap()
}

#[allow(dead_code)]
pub(crate) fn input_data_grain() -> Rc<ParsedShipData> {
    ParsedShipData::parse(
        multipler_x1::multipler_x1(),
        multipler_x2::multipler_x2(),
        multipler_s::multipler_s(),
        coefficient_k::coefficient_k(),
        coefficient_k_theta::coefficient_k_theta(),
        icing::icing(),
        1,
        ship::ship(),
        voyage::voyage(),
        ship_parameters::ship_parameters(),
        bounds::bounds(119.95, 59.194, 20),
        center_waterline::center_waterline(),
        waterline_length::waterline_length(),
        waterline_breadth::waterline_breadth(),
        waterline_area::waterline_area(),
        volume_shift::volume_shift(),
        rad_long::rad_long(),
        rad_trans::rad_trans(),
        h_subdivision::h_subdivision(),
        mean_draught::mean_draught(),
        center_draught_shift::center_draught_shift(),
        pantocaren::pantocaren(),
        flooding_angle::flooding_angle(),
        entry_angle::entry_angle(),
        bonjean_frame::bonjean_frame(),
        frame_area::frame_area(),
        draft_mark::draft_mark(),
        load_line::load_line(),
        screw::screw(),
        bow_board::bow_board(),
        LoadCargoArray {
            data: Vec::new(),
            error: HashMap::new(),
        },
        ContainerArray {
            data: Vec::new(),
            error: HashMap::new(),
        },
        bulkhead::bulkhead_27_51(),
        compartment::compartment_100_sea_grain(),
        hold_compartment::hold_compartment_grain(),
        load_constants::load_constants(),
        area_h_stab::area_h_stab(),
        area_h_str::area_h_str(),
        area_v_stab::area_v_stab(),
        area_v_str::area_v_str(),
        bow_area::bow_area(),
    )
    .unwrap()
}

#[allow(dead_code)]
pub(crate) fn input_data_19() -> Rc<ParsedShipData> {
    ParsedShipData::parse(
        multipler_x1::multipler_x1(),
        multipler_x2::multipler_x2(),
        multipler_s::multipler_s(),
        coefficient_k::coefficient_k(),
        coefficient_k_theta::coefficient_k_theta(),
        icing::icing(),
        1,
        ship::ship(),
        voyage::voyage(),
        ship_parameters::ship_parameters(),
        bounds::bounds(119.95, 59.194, 20),
        center_waterline::center_waterline(),
        waterline_length::waterline_length(),
        waterline_breadth::waterline_breadth(),
        waterline_area::waterline_area(),
        volume_shift::volume_shift(),
        rad_long::rad_long(),
        rad_trans::rad_trans(),
        h_subdivision::h_subdivision(),
        mean_draught::mean_draught(),
        center_draught_shift::center_draught_shift(),
        pantocaren::pantocaren(),
        flooding_angle::flooding_angle(),
        entry_angle::entry_angle(),
        bonjean_frame::bonjean_frame(),
        frame_area::frame_area(),
        draft_mark::draft_mark(),
        load_line::load_line(),
        screw::screw(),
        bow_board::bow_board(),
        LoadCargoArray {
            data: Vec::new(),
            error: HashMap::new(),
        },
        container::container_19(),
        bulkhead::bulkhead_27_28(),
        compartment::compartment_100_sea_19(),
        hold_compartment::hold_compartment_empty(),
        load_constants::load_constants(),
        area_h_stab::area_h_stab(),
        area_h_str::area_h_str(),
        area_v_stab::area_v_stab(),
        area_v_str::area_v_str(),
        bow_area::bow_area(),
    )
    .unwrap()
}
*/
