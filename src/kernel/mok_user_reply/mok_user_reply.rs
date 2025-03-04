use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_sync::services::entity::{cot::Cot, name::Name, object::Object, point::{point::Point, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status};
use serde::Serialize;
use tokio::task::JoinHandle;
use crate::{
    algorithm::entities::{
        bearing::Bearing, hoisting_rope::{hoisting_rope::HoistingRope, rope_durability_class::RopeDurabilityClass, rope_type::RopeType},
        hook::Hook,
    }, 
    infrostructure::client::{
        change_hoisting_tackle::{ChangeHoistingTackleQuery, ChangeHoistingTackleReply},
        choose_hoisting_rope::{ChooseHoistingRopeQuery, ChooseHoistingRopeReply},
        choose_user_bearing::{ChooseUserBearingQuery, ChooseUserBearingReply},
        choose_user_hook::{ChooseUserHookQuery, ChooseUserHookReply},
        query::Query
    },
    kernel::{str_err::str_err::StrErr, sync::link::Link},
};
///
/// Struct to imitate user's answer's
pub struct MokUserReply {
    dbg: String,
    txid: usize,
    name: Name,
    /// recieve and sender channel's
    link: Option<Link>,
    /// value to stop thread that await request's
    exit: Arc<AtomicBool>,
    exit_pair: Arc<AtomicBool>,
}
//
//
impl MokUserReply {
    ///
    /// Struct constructor
    pub fn new(parent: impl Into<String>, link: Link) -> Self {
        let name = Name::new(parent, "MokUserReply");
        let exit_pair = link.exit_pair();
        Self { 
            dbg: name.join(),
            txid: PointTxId::from_str(&name.join()),
            name: name,
            link: Some(link),
            exit: Arc::new(AtomicBool::new(false)),
            exit_pair,
        }
    }
    ///
    /// Starts service's main loop in the individual task
    pub async fn run(&mut self) -> Result<JoinHandle<()>, StrErr> {
        let mut link = self.link.take().unwrap_or_else(|| panic!("{}.run | Link not found", self.name));
        let dbg = self.name.join().clone();
        let txid = self.txid;
        log::info!("{}.run | Starting...", dbg);
        log::trace!("{}.run | Self tx_id: {}", dbg, PointTxId::from_str(self.id()));
        log::info!("{}.listen | Start", dbg);
        fn build_reply(dbg: &str, txid: usize, name: &str, reply: impl Serialize + std::fmt::Debug) -> Option<Point> {
            match serde_json::to_string(&reply) {
                Ok(reply) => Some(Point::String(PointHlr::new(
                    txid, name,
                    reply, Status::Ok, Cot::ReqCon,
                    chrono::offset::Utc::now(),
                ))),
                Err(err) => {
                    log::warn!("{}.listen | Serialize reply error: {:#?}, \n\tquery: {:#?}", dbg, err, reply);
                    None
                }
            }
        }
        let handle = link.listen(move |query: Point| {
            log::debug!("{}.listen | Received query: {:#?}", dbg, query);
            let name = query.name();
            let query = query.as_string().value;
            match serde_json::from_str::<Query>(query.as_str()) {
                Ok(query) => match query {
                    Query::ChooseUserHook(query) => {
                        let query: ChooseUserHookQuery = query;
                        let reply = match query.testing {
                            true => ChooseUserHookReply::new(Hook {
                                gost: "GOST 34567-85".to_string(),
                                r#type: "Forged".to_string(),
                                load_capacity_m13: 25.0,
                                load_capacity_m46: 23.0,
                                load_capacity_m78: 21.0,
                                shank_diameter: 85.0,
                                weight: 50.0,
                            }),
                            false => ChooseUserHookReply::new(Hook {
                                gost: "GOST 34567-85".to_string(),
                                r#type: "Forged".to_string(),
                                load_capacity_m13: 25.0,
                                load_capacity_m46: 23.0,
                                load_capacity_m78: 21.0,
                                shank_diameter: 85.0,
                                weight: 50.0,
                            }),
                        };
                        build_reply(&dbg, txid, &name, reply)
                    }
                    Query::ChooseUserBearing(query) => {
                        let _query: ChooseUserBearingQuery = query;
                        let reply = ChooseUserBearingReply::new(Bearing {
                            name: "8100H".to_owned(),
                            outer_diameter: 24.0,
                            inner_diameter: 10.0,
                            static_load_capacity: 11800.0,
                            height: 9.0,
                        });
                        build_reply(&dbg, txid, &name, reply)
                    }
                    Query::ChooseHoistingRope(query) => {
                        let _query: ChooseHoistingRopeQuery = query;
                        let reply = ChooseHoistingRopeReply::new(HoistingRope {
                            name: "STO 71915393-TU 051-2014 Octopus 826K".to_owned(),
                            rope_diameter: 12.0,
                            r#type: RopeType::Metal,
                            rope_durability: RopeDurabilityClass::C1770,
                            rope_force: 112.0,
                            s: 67.824,
                            m: 0.688,
                        });
                        build_reply(&dbg, txid, &name, reply)
                    }
                    Query::ChangeHoistingTackle(query) => {
                        let _query: ChangeHoistingTackleQuery = query;
                        let reply = ChangeHoistingTackleReply::new(1);
                        build_reply(&dbg, txid, &name, reply)
                    }
                }
                Err(err) => {
                    log::warn!("{}.listen | Deserialize error {:?} in {:#?}", dbg, err, query);
                    None
                }
            }
        }).await;
        // log::debug!("{}.run | Exit", dbg);
        handle
    }
    ///
    /// Sends "exit" signal to the service's thread
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        self.exit_pair.store(true, Ordering::SeqCst);
    }
}
//
//
impl Object for MokUserReply {
    fn id(&self) -> &str {
        &self.dbg
    }

    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl std::fmt::Debug for MokUserReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MokUserReply")
        .field("name", &self.name)
        // .field("link", &self.link)
        // .field("exit", &self.exit)
        .finish()
    }
}
