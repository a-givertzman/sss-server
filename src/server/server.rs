use std::{io::ErrorKind, net::TcpListener, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{collections::FxDashMap, sync::{Handles, Owner}, thread_pool::Scheduler};
use crate::{conf::Conf, kernel::{EvalEx, sync::Link}, server::{Connection, EvalResult, Event}};
use super::QueryId;
///
/// The Server
/// - Setups socket server at specified address
/// - Spawnes `Connection` on each incoming requiest
pub struct Server {
    dbg: Dbg,
    conf: Conf,
    scheduler: Scheduler,
    ctx: Arc<Box<dyn Fn(&Dbg, &Conf) -> Box<dyn EvalEx<(Event<QueryId>, Option<Link>), EvalResult<QueryId>> + Send > + Send + Sync>>,
    connections: Arc<FxDashMap<String, Connection>>,
    listener: Arc<Owner<Arc<TcpListener>>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
//
impl Server {
    ///
    /// Returns [Server] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: Conf,
        scheduler: Scheduler,
        ctx: impl Fn(&Dbg, &Conf) -> Box<dyn EvalEx<(Event<QueryId>, Option<Link>), EvalResult<QueryId>> + Send> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent.into(), "Server");
        Self {
            conf,
            scheduler,
            ctx: Arc::new(Box::new(ctx)),
            connections: Arc::new(FxDashMap::default()),
            listener: Arc::new(Owner::empty()),
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// [Server] Operation mode
    pub fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let conf = self.conf.clone();
        let scheduler = self.scheduler.clone();
        let ctx = self.ctx.clone();
        let connections = self.connections.clone();
        let listeners = self.listener.clone();
        let exit = self.exit.clone();
        let handle = self.scheduler.spawn(move || {
            'main: while !exit.load(Ordering::Acquire) {
                match TcpListener::bind(conf.server.address.clone()) {
                    Ok(listener) => {
                        if let Err(err) = listener.set_nonblocking(true) {
                            log::warn!("{dbg}.wait | Can't switch TcpListener to nonblocking, error: {:?}", err);
                        }
                        let listener= Arc::new(listener);
                        listeners.replace(listener.clone());
                        for stream in listener.incoming() {
                            match stream {
                                Ok(stream) => {
                                    let client = stream.peer_addr().map(|a| a.to_string()).unwrap_or(connections.len().to_string());
                                    let conn = Connection::new(
                                        &dbg,
                                        conf.server.connection.clone(),
                                        stream,
                                        scheduler.clone(),
                                        (ctx)(&dbg, &conf),
                                    );
                                    match conn.run() {
                                        Ok(_) => _ = connections.insert(client, conn),
                                        Err(err) => log::warn!("{dbg}.run | Spawn connection error: {:?}", err),
                                    }
                                    let keys: Vec<String> = connections.iter().map(|e| e.key().clone()).collect();
                                    for key in keys {
                                        if let Some(con) =  connections.get(&key) {
                                            if con.value().is_finished() {
                                                con.exit();
                                                connections.remove(&key);
                                            }
                                        }
                                    }
                                }
                                Err(err) => if !exit.load(Ordering::Acquire) {
                                    if err.kind() != ErrorKind::WouldBlock {
                                        log::warn!("{dbg}.run | Can't get incoming TcpStream, error: {:?}", err)
                                    }
                                }
                            }
                            if exit.load(Ordering::Acquire) {
                                break 'main;
                            }
                            // Normal operation timeout between incoming connections
                            std::thread::sleep(Duration::from_millis(100));
                        }
                    }
                    Err(err) => log::warn!("{dbg}.run | Bind TcpServer error: {:?}", err),
                }
                // Normal operation timeout before next time bind server socket
                std::thread::sleep(Duration::from_secs(1));
            }
            log::debug!("{dbg}.run | Exit");
            Ok(())
        }).map_err(|err| Error::new(&self.dbg, "run").pass(err))?;
        self.handles.push(handle);
        Ok(())
    }
    ///
    /// Returns when internal thread's will finished
    #[allow(unused)]
    pub fn wait(&self) -> Result<(), Error> {
        for conn in self.connections.iter() {
            log::debug!("{}.wait | Wait for Connection '{}'...", self.dbg, conn.key());
            if let Err(err) = conn.wait() {
                log::warn!("{}.wait | Wait for Connection '{}' error: {:?}", self.dbg, conn.key(), err);
            }
        }
        self.handles.wait()
    }
    ///
    /// Sends exit signal to main tread
    #[allow(unused)]
    pub fn exit(&self) {
        self.exit.store(true, Ordering::Release);
        for conn in self.connections.iter() {
            conn.exit();
        }
    }
}
