#![allow(non_snake_case)]

use std::{net::{TcpStream, SocketAddr, ToSocketAddrs}, time::Duration, sync::{Arc, Mutex, mpsc::{Sender, Receiver, self}}, thread};

use log::{warn, LevelFilter, debug, info};


// #[derive(Debug, PartialEq)]
// enum ConnectState {
//     Closed,
//     Connecting,
//     Connected,
// }
// impl ConnectState {
//     fn from(value: usize) -> Self {
//         match value {
//             0 => ConnectState::Closed,
//             1 => ConnectState::Connecting,
//             2 => ConnectState::Connected,
//             _ => panic!("Invalid value: '{}'", value)
//         }
//     }
//     fn value(&self) -> usize {
//         match self {
//             ConnectState::Closed => 0,
//             ConnectState::Connecting => 1,
//             ConnectState::Connected => 2,
//         }
//     }
// }



///
/// Opens a TCP connection to a remote host
/// - returns connected Result<TcpStream, Err>
pub struct TcpClientConnect {
    id: String,
    addr: SocketAddr,
    stream: Arc<Mutex<Vec<TcpStream>>>,
    reconnect: Duration,
    exitSend: Sender<bool>,
    exitRecv: Arc<Mutex<Receiver<bool>>>,
}
///
/// Opens a TCP connection to a remote host
impl TcpClientConnect {
    ///
    /// Creates a new instance of TcpClientConnect
    pub fn new(parent: impl Into<String>, addr: impl ToSocketAddrs + std::fmt::Debug, reconnect: Duration) -> TcpClientConnect {
        let addr = match addr.to_socket_addrs() {
            Ok(mut addrIter) => {
                match addrIter.next() {
                    Some(addr) => addr,
                    None => panic!("TcpClientConnect({}).connect | Empty address found: {:?}", parent.into(), addr),
                }
            },
            Err(err) => panic!("TcpClientConnect({}).connect | Address parsing error: \n\t{:?}", parent.into(), err),
        };
        let (send, recv) = mpsc::channel();
        Self {
            id: format!("{}/TcpClientConnect", parent.into()),
            addr,
            stream: Arc::new(Mutex::new(Vec::new())),
            reconnect,
            exitSend: send,
            exitRecv: Arc::new(Mutex::new(recv)),
        }
    }
    ///
    /// Opens a TCP connection to a remote host until succeed.
    pub fn connect(&mut self) -> Option<TcpStream> {
        info!("TcpClientConnect({}).connect | connecting...", self.id);
        let id = self.id.clone();
        let addr = self.addr.clone();
        info!("TcpClientConnect({}).inner_connect | connecting to: {:?}...", id, addr);
        let cycle = self.reconnect;
        let selfStream = self.stream.clone();
        let exit = self.exitRecv.clone();
        let handle = thread::spawn(move || {
            let exit = exit.lock().unwrap();
            loop {
                cycle.start();
                match TcpStream::connect(addr) {
                    Ok(stream) => {
                        selfStream.lock().unwrap().push(stream);
                        info!("TcpClientConnect({}).inner_connect | connected to: \n\t{:?}", id, selfStream.lock().unwrap().first().unwrap());
                        break;
                    },
                    Err(err) => {
                        if log::max_level() == LevelFilter::Debug {
                            warn!("TcpClientConnect({}).inner_connect | connection error: \n\t{:?}", id, err);
                        }
                    }
                };
                match exit.try_recv() {
                    Ok(exit) => {
                        debug!("TcpClientConnect({}).inner_connect | exit: {}", id, exit);
                        if exit {
                            break;
                        }
                    },
                    Err(_) => {},
                }
                cycle.wait();
            }
            debug!("TcpClientConnect({}).inner_connect | exit", id);
        });
        handle.join().unwrap();
        let mut tcpStream = self.stream.lock().unwrap();
        tcpStream.pop()
    }
    ///
    /// Opens a TCP connection to a remote host with a timeout.
    pub fn connect_timeout(&self, timeout: Duration) -> Result<TcpStream, std::io::Error> {
        TcpStream::connect_timeout(&self.addr, timeout)
    }
    ///
    /// Exit thread
    pub fn exit(&self) -> Sender<bool> {
        self.exitSend.clone()
    }
}