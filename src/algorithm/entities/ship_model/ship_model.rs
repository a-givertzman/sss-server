use crate::algorithm::entities::Curve;
use crate::algorithm::entities::Position;
use crate::algorithm::entities::data::PointDataArray;
use crate::algorithm::entities::data::serde_parser::IFromJson;
use crate::algorithm::entities::data::stability::horizontal_area::HStabArea;
use crate::algorithm::entities::data::stability::horizontal_area::HStabAreaArray;
use crate::algorithm::entities::data::strength::horizontal_area::HStrArea;
use crate::algorithm::entities::data::strength::horizontal_area::HStrAreaArray;
use crate::algorithm::entities::data::strength::physical_frame::PhysicalFrameArray;
use crate::algorithm::entities::model_cached::AreaResult;
use crate::algorithm::entities::model_cached::DsoResult;
use crate::algorithm::entities::model_cached::ModelCached;
use crate::algorithm::entities::ship_model::grain_moment::GrainMoment;
use crate::algorithm::entities::ship_model::grain_moment::GrainMomentDataArray;
use crate::algorithm::entities::ship_model::stability_result::BalanceStabilityResult;
use crate::algorithm::entities::ship_model::volume_max::MaxDataArray;
use crate::algorithm::entities::ship_model::*;
use crate::algorithm::entities::{Bound, Bounds};
use crate::algorithm::eval::strength::StrengthBalanceCtx;
use crate::infrostructure::ApiClient;
use sal_core::dbg::Dbg;
use sal_core::error::Error;
use std::collections::HashMap;
use std::{fmt::Debug, sync::Arc};
///
///
pub struct ShipModel {
    //   txid: usize,
    //   name: Name,
    dbg: Dbg,
    ship_id: String,
    //   ship_file_name: String, // TODO - read by  ship_id
    project_id: String,
    bounds: Option<Bounds>,
    horisontal_area_str: Option<Vec<f64>>,
    horisontal_area_stab: Option<f64>,
    horisontal_area_shift: Option<Position>,
    windage_area_stab: Option<f64>,
    windage_area_moment: Option<Moment>,
    grain_moments: Option<HashMap<String, GrainMoment>>,
    opening: Option<Vec<Position>>,
    deck_angle_point: Option<Vec<Position>>,
    model_cached: ModelCached,
    //    timeout: Duration,
    api_client: Arc<ApiClient>,
}
//
//
impl ShipModel {
    ///
    /// Default timeout to await `recv`` operation, 300 ms
    //   const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10);
    /// TODO
    /// Returns [ShipModel] new instance
    /// - `send` - local side of channel.send
    /// - `recv` - local side of channel.recv
    /// - `exit` - exit signal for `recv_query` method
    pub fn new(
        parent: impl Into<String>,
        ship_id: String,
        //    ship_file_name: String,
        project_id: String,
        model_cached: ModelCached,
        api_client: Arc<ApiClient>,
    ) -> Self {
        //    let name = Name::new(parent, "ShipModel");
        let dbg = Dbg::new(parent, "ShipModel");
        Self {
            //    txid: PointTxId::from_str(&name.join()),
            //    name,
            dbg,
            ship_id,
            //      ship_file_name,
            project_id,
            bounds: None,
            horisontal_area_str: None,
            horisontal_area_stab: None,
            horisontal_area_shift: None,
            windage_area_stab: None,
            windage_area_moment: None,
            grain_moments: None,
            opening: None,
            deck_angle_point: None,
            model_cached: model_cached,
            //    timeout: Self::DEFAULT_TIMEOUT,
            api_client,
        }
    }
    /// TODO - Doc
    /// TODO - сделать что-то с Bounds, они нужны в модели и в контексте
    /// ПО идее их должен читать контекст, но модель инициализируется первее
    pub fn init(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "init");
        self.grain_moments = Some(
            grain_moments(
                &self.ship_id,
                &self.project_id,
                &self.api_client,
            )
            .map_err(|err| error.pass(err))?,
        );
        self.opening = Some(
            opening(
                &self.ship_id,
                &self.project_id,
                &self.api_client.clone(),
            )
            .map_err(|err| error.pass(err))?,
        );
        self.deck_angle_point = Some(
            deck_angle_point(
                &self.ship_id,
                &self.project_id,
                &self.api_client,
            )
            .map_err(|err| error.pass(err))?,
        );
        // TODO переделать, пока не понятно в какой момент должны читаться объемы
        // возможно их надо пересчитывать каждый расчет
        let compartments_max = compartments_max(
            &self.ship_id,
            &self.project_id,
            &self.api_client,
        )
        .map_err(|err| error.pass_with("compartments_max", err))?;
        let bounds = get_physical_bounds(
            &self.ship_id, 
            &self.project_id, 
            &self.api_client,
        )
        .map_err(|err| error.pass_with("bounds", err))?;
        self.horisontal_area_str = {
            let horisontal_area = horisontal_area_str(
                &self.ship_id,
                &self.project_id,
                &self.api_client,
            )
            .map_err(|err| error.pass(err))?;
            let (horisontal_area, errors): (Vec<_>, Vec<_>) = horisontal_area
                .into_iter()
                .map(|v| {
                    let bound = match Bound::new(v.bound_x1, v.bound_x2) {
                        Ok(bound) => bound,
                        Err(err) => {
                            return Err(
                                error.pass_with(format!("Bound::new, name:{}", v.name), err)
                            );
                        }
                    };
                    Ok((v.value, bound))
                })
                .partition(|v| v.is_ok());
            if !errors.is_empty() {
                return Err(error.pass_with(
                    "horisontal_area_str",
                    errors.iter().fold(String::new(), |acc, err| {
                        format!("{acc}\n\t error: {:?}", err)
                    }),
                ));
            }
            let mut horisontal_area_values = vec![0.; bounds.len_qnt()];
            for current in horisontal_area.into_iter() {
                match current {
                    Ok((area, src_bound)) => {
                        bounds.iter().enumerate().for_each(|(i, &trg_bound)| {
                            horisontal_area_values[i] +=
                                area * src_bound.part_ratio(&trg_bound).unwrap_or(0.)
                        });
                    }
                    Err(err) => return Err(error.pass_with("horisontal_area", err)),
                }
            }
            Some(horisontal_area_values)
        };
        let (horisontal_area_stab, horisontal_area_moment) = {
            let horisontal_area_stab = horisontal_area_stab(
                &self.ship_id,
                &self.project_id,
                &self.api_client,
            )
            .map_err(|err| error.pass(err))?;
            horisontal_area_stab
                .into_iter()
                .fold((0., Moment::zero()), |(sum_a, sum_m), v| {
                    (
                        sum_a + v.value,
                        sum_m
                            + Moment::from_pos(
                                Position::new(v.shift_x, v.shift_y, v.shift_z),
                                v.value,
                            ),
                    )
                })
        };
        self.horisontal_area_stab = Some(horisontal_area_stab);
        self.horisontal_area_shift = Some(horisontal_area_moment.to_pos(horisontal_area_stab));
        self.bounds = Some(bounds.clone());
        self.model_cached
            .init(compartments_max, &bounds)
            .map_err(|err| Error::new(&self.dbg, "init").pass(err))?;
        let windage = self
            .model_cached
            .windage_area_min()
            .map_err(|err| Error::new(&self.dbg, "init").pass(err))?;
        self.windage_area_stab = Some(windage.0);
        self.windage_area_moment = Some(Moment::new(windage.1, 0., windage.2));
        self.bounds = Some(bounds);
        Ok(())
    }
    ///
    pub fn update_hold_compartments(
        &mut self,
        new_hold_compartments: &Vec<(String, Vec<String>)>,
    ) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "update_hold_compartments");
        {
            let mut grain_moments = self
                .grain_moments
                .take()
                .ok_or(error.err("grain_moments"))?;

            for (code, part_codes) in new_hold_compartments {
                if grain_moments.contains_key(code) {
                    continue;
                }
                let new_grain_moment = GrainMoment::new(
                    part_codes
                        .iter()
                        .filter_map(|code| grain_moments.get(code))
                        .fold(Vec::new(), |mut acc, v| {
                            acc.append(&mut v.curves());
                            acc
                        }),
                );
                grain_moments.insert(code.to_owned(), new_grain_moment);
            }
            self.grain_moments = Some(grain_moments);
        }
        self.model_cached
            .update_hold_compartments(new_hold_compartments)
    }
    ///
    /// TODO: Doc
    pub fn bounds(&self) -> Result<Bounds, Error> {
        self.bounds.clone().ok_or(Error::new(&self.dbg, "bounds"))
    }
    /*
         match &self.bounds {
               Some(bounds) => Ok(bounds.clone()),
               None => match get_bounds(
                   &self.api_client.read(),
                   self.ship_id,
                   self.project_id.clone(),
                   qnt_bounds,
               ) {
                   Ok(bounds) => {
                       self.bounds = Some(bounds.clone());
                       Ok(bounds)
                   },
                   Err(_) => {
                       let bounds = self.model_cached.rebuild_bounds(qnt_bounds).map_err(|err| error.pass_with("self.model_cached.rebuild_bounds", err))?;
                       self.bounds = Some(bounds.clone());
                       let sql = format!("INSERT INTO computed_frame_space\n\t(ship_id, qnt_bounds, index, start_x, end_x)\nVALUES{};",
                           bounds.iter().filter(|v| v.is_value()).enumerate().map(|(i, v)| format!("\n\t({}, '{}', {i}, {}, {})", self.ship_id, qnt_bounds, v.start().unwrap(), v.end().unwrap())).collect::<Vec<_>>().join(","));
                       self.api_client.read().fetch(&sql).map_err(|err| error.pass_with("self.api_client.fetch", err))?;
                       Ok(bounds)
                   }
               }
           }
       }
    */
    ///
    /// Разбиение площадей поверхности корпуса по шпациям для расчета прочности
    pub fn strength_area(&self) -> Result<StrengthArea, Error> {
        let error = Error::new(&self.dbg, "strength_area");
        let windage_area = self
            .model_cached
            .bounded_windage_area()
            .map_err(|err| error.pass_with("model_cached.bounded_windage_area", err))?;
        let horisontal_area = self
            .horisontal_area_str
            .clone()
            .ok_or(error.err("no horisontal_area"))?;
        /*     println!("\nhorisontal_area_values\n");
                horisontal_area_values.iter().for_each(|v| print!(" {:.3}", v));
                println!("\nwindage_area\n");
                windage_area.iter().for_each(|v| print!(" {:.3}", v));
        */
        Ok(StrengthArea {
            v: windage_area,
            h: horisontal_area,
        })
    }
    /// Площади и моменты площади горизонтальных поверхностей корпус.
    /// Возвращает площадь поверхностей + момент площади
    pub fn static_area_h(&self) -> Result<(f64, Position), Error> {
        let error = Error::new(&self.dbg, "static_area_h");
        let area = self
            .horisontal_area_stab
            .clone()
            .ok_or(error.err("no horisontal_area_stab"))?;
        let shift = self
            .horisontal_area_shift
            .clone()
            .ok_or(error.err("no horisontal_area_shift"))?;
        Ok((area, shift))
    }
    /// Площади и моменты поверхности парусности корпуса.
    /// Возвращает площадь поверхностей + момент площади
    pub fn static_area_v(&self) -> Result<(f64, Moment), Error> {
        let error = Error::new(&self.dbg, "static_area_v");
        let area = self
            .horisontal_area_stab
            .clone()
            .ok_or(error.err("no horisontal_area_stab"))?;
        let moment = self
            .horisontal_area_shift
            .clone()
            .ok_or(error.err("no horisontal_area_moment"))?;
        Ok((area, moment))
    }
    ///
    /// Площади и моменты поверхности корпуса для расчета остойчивости
    pub fn stability_area(&self, draught_mid: f64) -> Result<StabilityArea, Error> {
        let error = Error::new(&self.dbg, "stability_area");
        let AreaResult {
            av_cs_dmin,
            mv_x_cs_dmin,
            mv_z_cs_dmin,
            delta_av,
            delta_mv_x,
            delta_mv_z,
            area_volume_z,
        } = self
            .model_cached
            .windage_area(draught_mid)
            .map_err(|err| error.pass_with("model_cached.windage_area", err))?;
        Ok(StabilityArea {
            av_cs_dmin1: av_cs_dmin,
            mv_x_cs_dmin1: mv_x_cs_dmin,
            mv_z_cs_dmin1: mv_z_cs_dmin,
            delta_av,
            delta_mv_x,
            delta_mv_z,
            area_volume_z,
        })
    }
    ///
    /// TODO: Doc
    pub fn compute_stability(
        &self,
        query: BalanceStabilityQuery,
    ) -> Result<BalanceStabilityResult, Error> {
        let error = Error::new(&self.dbg, "compute_balance");
        let mut result = self
            .model_cached
            .balance_stability(&query, 0.000001)
            .map_err(|err| error.pass(err))?;
        let grain_moments = self
            .grain_moments
            .as_ref()
            .ok_or(error.err("grain_moment"))?;
        // TODO - переписать получение момента из модели
        result.bulk.iter_mut().for_each(|v| {
            if !v.shiftable {
                v.grain_moment = 0.;
                return;
            }
            if let Some(curve) = grain_moments.get(&v.code) {
                v.grain_moment = curve.value(v.level).unwrap_or(0.);
                return;
            }
            let error = error.err(format!("grain_moments.get(&v.code), {}", v.code));
            log::error!("{}", error);
            v.grain_moment = 0.;
        });
        Ok(result)
    }
    ///
    /// TODO: Doc
    pub fn compute_strength(
        &self,
        query: BalanceStrengthQuery,
    ) -> Result<StrengthBalanceCtx, Error> {
        self.model_cached
            .balance_strength(query)
            .map_err(|err| Error::new(&self.dbg, "compute_strength").pass(err))
    }
    ///
    /// TODO: Doc
    pub fn compute_dso(
        &self,
        heel: f64,
        trim: f64,
        draught_mid: f64,
        cg: Position,
        query: BalanceStabilityQuery,
    ) -> Result<DsoResult, Error> {
        let error = Error::new(&self.dbg, "compute_balance");
        let opening = self.opening.as_ref().ok_or(error.err("opening"))?;
        let deck_angle_point = self
            .deck_angle_point
            .as_ref()
            .ok_or(error.err("deck_angle_point"))?;
        let result = self
            .model_cached
            .dso(
                heel,
                trim,
                draught_mid,
                cg,
                query,
                opening,
                deck_angle_point,
                0.001,
            )
            .map_err(|err| error.pass(err))?;
        Ok(result)
    }
}
//
//
impl Debug for ShipModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShipModel")
            .field("ship_id", &self.ship_id)
            .field("project_id", &self.project_id)
            // .field("send", &self.send)
            // .field("recv", &self.recv)
            // .field("subscribers", &self.subscribers)
            // .field("receivers", &self.receivers)
            //   .field("timeout", &self.timeout)
            //   .field("exit", &self.exit)
            .finish()
    }
}
///
/// Получение шпаций из физических фреймов
fn get_physical_bounds(
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
) -> Result<Bounds, Error> {
    let error = Error::new("ShipModel", "get_physical_bounds");
    let data = api_client.fetch(&format!(
                "SELECT pos_x, frame_index as index FROM physical_frame WHERE ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY index ASC;"
            )).map_err(|err| error.pass(err))?;
    let physical_frames: Vec<_> = PhysicalFrameArray::parse(&data)
        .map_err(|err| error.pass(err))?
        .data();
    let bounds = Bounds::from_array(&physical_frames, 0.).map_err(|err| error.pass(err))?;
    Ok(bounds)
}
// временные функции пока непонятно как работать с базой при изменении данных
// TODO - перенести все в контекст
/*
///
/// Получение шпаций, вероятно не нужно, шпации будут считаться из физических фреймов
fn get_bounds(
    api_client: &ApiClient,
    ship_id: &str,
    project_id: String,
    qnt_bounds: usize,
) -> Result<Bounds, Error> {
    let error = Error::new("ShipModel", "get_bounds");
    let data = api_client.fetch(&format!(
            "SELECT index, start_x, end_x FROM computed_frame_space WHERE qnt_bounds = {qnt_bounds} AND ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY index ASC;"
        ));
    let bounds = match data {
        Ok(data) => {
            // если шпации для разбиения на qnt_bounds есть, значит есть кэш для этого разбиения - читаем их
            match ComputedFrameDataArray::parse(&data) {
                Ok(data) => data.data(),
                Err(err) => return Err(error.pass_with("parse error", err)),
            }
        }
        Err(err) => return Err(err),
        // если шпаций для qnt_bounds нет,
        // значит нет кэшей для такого разбиения и надо их пересчитать
        /*    {
           // TODO: если шпаций для qnt_bounds нет, значит создаем такое разбиение и считаем кэш
            let data = api_client.fetch(&format!(
                "SELECT pos_x, frame_index as index FROM physical_frame WHERE ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY index ASC;"
            ));
            match data {
                Ok(data) => {
                    let physical_frames = match PhysicalFrameArray::parse(&data) {
                        Ok(data) => data.data(),
                        Err(err2) => {
                            return Err(error.pass(format!(
                                "error: {err1}, physical_frames parse error: {err2}"
                            )));
                        }
                    };
                    if let (Some(bow_x), Some(stern_x)) =
                        (physical_frames.first(), physical_frames.last())
                    {
                        return match Bounds::from_min_max(stern_x.1, bow_x.1, qnt_bounds) {
                            Ok(bounds) => Ok(bounds),
                            Err(err2) => {
                                return Err(error
                                    .pass(format!("error: {err1}, create_bounds error: {err2}")));
                            }
                        };
                    } else {
                        return Err(error.pass(format!("error: {err1}, can't get bow_x, stern_x!")));
                    };
                }
                Err(err2) => {
                    return Err(error.pass(format!("error: {err1}, physical_frames error: {err2}")));
                }
            };
        }*/
    };
    let bounds: Bounds = match Bounds::from_frames(&bounds) {
        Ok(data) => data,
        Err(err) => return Err(error.pass(err)),
    };
    Ok(bounds)
}
*/
/// Чтение данных горизонтальных поверхностей для прочности из базы
fn horisontal_area_str(
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
) -> Result<Vec<HStrArea>, Error> {
    let err = Error::new("ShipModel", "horisontal_area_str");
    let area = HStrAreaArray::parse(
        &api_client.fetch(&format!(
            "SELECT name, value, bound_x1, bound_x2 FROM \"ship/ship_structures/area/h_str\" WHERE ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id} ORDER BY bound_x1 ASC;"
        )).map_err(|e| err.pass(e.to_string()))?
    ).map_err(|e| err.pass(e.to_string()))?;
    Ok(area.data())
}
/// Чтение данных горизонтальных поверхностей для остойчивости из базы
fn horisontal_area_stab(
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
) -> Result<Vec<HStabArea>, Error> {
    let err = Error::new("ShipModel", "horisontal_area_stab");
    let area = HStabAreaArray::parse(&api_client.fetch(
        &format!("SELECT name, value, shift_x, shift_y, shift_z FROM \"ship/ship_structures/area/h_stab\" WHERE ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id};")
    )?).map_err(|e| err.pass(e.to_string()))?;
    Ok(area.data())
}
/// Чтение данных объемного кренящего момента для зерна.
/// Возвращает мапу (ид отсека, кривая момента от уровня заполнения отсека)
fn grain_moments(
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
) -> Result<HashMap<String, GrainMoment>, Error> {
    let error = Error::new("ShipModel", "grain_moments");
    let sql = format!(
        "SELECT code, level, moment FROM grain_moment_view WHERE ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id};"
    );
    let data = GrainMomentDataArray::parse(
        &api_client.fetch(&sql).map_err(|err| error.pass_with(format!("api_client.fetch {sql}"), err))?
    ).map_err(|err| error.pass_with("parse", err))?;
    let data: Vec<(String, Result<Curve<f64>, Error>)> = data
        .data()
        .iter()
        .map(|(code, v)| (code.clone(), Curve::new_linear(v)))
        .collect();
    if let Some(error_data) = data.iter().filter(|v| v.1.is_err()).next() {
        error_data
            .1
            .clone()
            .map_err(|err| error.pass_with("Curve::new_linear", err))?;
    }
    let mut result = HashMap::new();
    for data in data.into_iter() {
        let curve = data.1.map_err(|err| error.pass(err))?;
        result.insert(data.0, GrainMoment::new(vec![curve]));
    }
    Ok(result)
}
/// Чтение максимального объема и уровня для отсеков
/// Возвращает мапу (ид отсека, максимальный объем (нетто))
fn compartments_max(
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
) -> Result<HashMap<String, (Option<f64>, f64)>, Error> {
    let error = Error::new("ShipModel", "compartments_max");
    let data = MaxDataArray::parse(
        &api_client
            .fetch(&format!(
                "SELECT
                s.code as code, \
                c.level_max as level_max,
                c.volume_max as volume_max
            FROM
                \"space\" AS s 
            INNER JOIN 
                \"space/compartment\" AS c ON s.compartment_id = c.id 
            WHERE ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id};"
            ))
            .map_err(|err| error.pass_with("api_client.fetch", err))?,
    )
    .map_err(|err| error.pass_with("parse", err))?;
    Ok(data.data())
}
/// Чтение таблицы угла входа в воду кромки палубы
fn deck_angle_point(
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
) -> Result<Vec<Position>, Error> {
    let error = Error::new("ShipModel", "deck_angle_point");
    let data = PointDataArray::parse(
        &api_client
            .fetch(&format!(
                "SELECT 
                    value_x as point_x, \
                    value_y as point_y, \
                    value_z as point_z
                FROM 
                    \"ship/ship_structures/deck_angle_point\" 
                WHERE 
                    ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id};"
            ))
            .map_err(|err| error.pass_with("deck_angle_point", err))?,
    )
    .map_err(|err| error.pass_with("parse", err))?;
    Ok(data.data())
}
/// Чтение таблицы открытых отверстий
fn opening(
    ship_id: &str,
    project_id: &str,
    api_client: &ApiClient,
) -> Result<Vec<Position>, Error> {
    let error = Error::new("ShipModel", "opening");
    let data = PointDataArray::parse(
        &api_client
            .fetch(&format!(
                "SELECT 
                    value_x as point_x, \
                    value_y as point_y, \
                    value_z as point_z
                FROM 
                    \"ship/ship_structures/opening\" 
                WHERE 
                    ship_id={ship_id} AND project_id IS NOT DISTINCT FROM {project_id};"
            ))
            .map_err(|err| error.pass_with("opening", err))?,
    )
    .map_err(|err| error.pass_with("parse", err))?;
    Ok(data.data())
}
