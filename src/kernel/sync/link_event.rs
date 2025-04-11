use std::{any::Any, fmt::Debug};

#[derive(Debug)]
pub struct Event {
    kind: EventKind,
    query: Box<dyn Any>,
    reply: Box<dyn Any>,
}
impl Event {
    pub fn query<T: Clone + 'static>(&self) -> T {
        let val = match self.query.downcast_ref::<T>() {
            Some(val) => val.to_owned(),
            None => todo!(),
        };
        val
    }
    pub fn reply<T: Clone + 'static>(&self) -> T {
        let val = match self.reply.downcast_ref::<T>() {
            Some(val) => val.to_owned(),
            None => todo!(),
        };
        val
    }
}

// pub(super) trait EventQuery: Debug {
//     fn query<T: Sized>(&self) -> T;
// }
// pub(super) trait EventReply: Debug {
//     fn reply<T: Sized>(&self) -> T;
// }

// impl EventQuery for Event {
//     fn query<T>(&self) -> T {
//         self.query
//     }
// }
// impl EventReply for Event {
//     fn reply<T>(&self) -> T {
//         self.reply
//     }
// }


#[derive(Debug)]
enum EventKind {
    Query,
    Reply,
}