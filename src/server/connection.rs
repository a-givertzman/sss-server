use std::{io::{BufReader, BufWriter, Read, Write}, net::{Shutdown, TcpStream}, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{sync::{Handles, Owner}, thread_pool::Scheduler};
use crate::{
    kernel::{EvalEx, sync::{Hub, Link}},
    server::{ConnectionConf, Content, Cot, EvalResult, Field, FieldConf, FieldId, FindField, FixedField, Message, QueryId, Reply, SizedField, Terminator}
};
use super::{Event, Response};

///
/// The [Connection] of the `Server`
pub struct Connection {
    dbg: Dbg,
    conf: ConnectionConf,
    stream: Owner<TcpStream>,
    scheduler: Scheduler,
    ctx: Owner<Box<dyn EvalEx<(Event<QueryId>, Option<Link>), EvalResult<QueryId>> + Send>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl Connection {
    ///
    /// Returns [Connection] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: ConnectionConf,
        stream: TcpStream,
        scheduler: Scheduler,
        ctx: Box<dyn EvalEx<(Event<QueryId>, Option<Link>), EvalResult<QueryId>> + Send + 'static>,
    ) -> Self {
        let dbg = Dbg::new(parent.into(), "Connection");
        Self {
            conf,
            stream: Owner::new(stream),
            scheduler,
            ctx: Owner::new(ctx),
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Setups TCP [Message]
    pub(crate) fn tcp_message(dbg: &Dbg) -> Message<((((((((), ()), ()), FieldId), Content), Cot), QueryId), u32), Vec<u8>> {
        const SYN: u8 = 0x22;
        Message::new(
            dbg, // Start |  Id   | Kind | Cot  |  Size  | Data
            vec![
                FieldConf::Const(vec![SYN]),    // Syn
                FieldConf::U32Be,               // ID
                FieldConf::Byte,                // Kind
                FieldConf::Byte,                // Cot
                FieldConf::U32Be,               // QueryId
                FieldConf::U32Be,               // Size
                FieldConf::Bytes,               // Payload bytes
            ],
            SizedField::new(
                dbg,
                |_, size| *size as usize,
                |_, bytes| {
                    Ok(bytes.to_vec())
                },
                FixedField::new(        // Size | u32
                    dbg,
                    4,
                    |dbg, bytes| {
                        match bytes.try_into() {
                            Ok(bytes) => Ok(u32::from_be_bytes(bytes)),
                            Err(err) => Err(Error::new(dbg, "Id::from_bytes").pass_with(format!("Can't parse 'Size' u32 filed from bytes {:?}", bytes), format!("{err}"))),
                        }
                    },
                    FixedField::new(        // Query | u32
                        dbg,
                        4,
                        |dbg, bytes| {
                            match QueryId::from_be_bytes(bytes) {
                                Ok(query) => Ok(query),
                                Err(err) => Err(Error::new(dbg, "Id::from_bytes").pass_with(format!("Can't parse 'Query' u32 filed"), err)),
                            }
                        },
                        FixedField::new(        // Cot | u8
                            dbg,
                            1,
                            |dbg, bytes| {
                                match Cot::from_be_bytes(bytes) {
                                    Ok(cot) => Ok(cot),
                                    Err(err) => Err(Error::new(dbg, "Cot::from_bytes").pass_with("Can't parse 'Cot' u8 field", err)),
                                }
                            },
                            FixedField::new(        // Kind | u8
                                dbg,
                                1,
                                |dbg, bytes| {
                                    match Content::try_from(bytes) {
                                        Ok(kind) => Ok(kind),
                                        Err(err) => Err(Error::new(dbg, "Kind::from_bytes").pass_with("Can't parse 'Kind' u8 field", err)),
                                    }
                                },
                                FixedField::new(        // ID | u32
                                    dbg,
                                    4,
                                    |dbg, bytes| {
                                        match FieldId::from_be_bytes(bytes) {
                                            Ok(id) => Ok(id),
                                            Err(err) => Err(Error::new(dbg, "Id::from_bytes").pass_with("Can't parse 'ID' u32 filed", err)),
                                        }
                                    },
                                    FindField::new(        // SYN | u8
                                        dbg,
                                        1,
                                        |_, bytes| {
                                            match bytes {
                                                [SYN] | [SYN, ..] => Ok(Some(())),
                                                _ => Ok(None)
                                            }
                                        },
                                        Terminator::new(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                )
            )
        )
    }
    ///
    /// - Sets the read timeout to the timeout specified.
    /// - Sets the write timeout to the timeout specified.
    fn set_tcp_timeout(&self, stream: &TcpStream, timeout: Duration) {
        if let Err(err) = stream.set_read_timeout(Some(timeout)) {
            log::warn!("{}.set_tcp_timeout | Set read timeout error: {:?}", self.dbg, err);
        }
        if let Err(err) = stream.set_write_timeout(Some(timeout)) {
            log::warn!("{}.set_tcp_timeout | Set write timeout error: {:?}", self.dbg, err);
        }
    }
    ///
    /// [Connection] Operation mode
    pub fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let error = Error::new(&self.dbg, "run");
        let conf = self.conf.clone();
        let stream = self.stream.take().ok_or(error.err("Can't take stream"))?;
        self.set_tcp_timeout(&stream, conf.timeout.to_duration());
        let ctx = self.ctx.take().ok_or(error.err("Can't take ctx"))?;
        let hub = Arc::new(Hub::new(&dbg, Some(self.exit.clone())));
        self.send(&stream, hub.clone())?;
        let exit = self.exit.clone();
        let handle = self.scheduler.spawn(move || {
            let error = Error::new(&dbg, "run");
            let link = hub.link();
            let mut r_stream = BufReader::new(&stream);
            let mut message = Self::tcp_message(&dbg);
            let mut buf = [0u8; 1024 * 4];
            while !exit.load(Ordering::Acquire) {
                match r_stream.read(&mut buf) {
                    Ok(len) => {
                        match message.parse(buf[..len].to_owned()) {
                            Ok(((((((_, FieldId(event_id)), content), cot), query_id), _), bytes)) => {
                                let response = match ctx.eval((Event::new(event_id, query_id, cot, content, bytes), Some(hub.link()))) {
                                    Ok(response) => response,
                                    Err(err) => Some(Response {
                                        event_id,
                                        query_id,
                                        cot: cot.reply_err(),
                                        reply: Reply::error(error.pass(err).to_string()),
                                    }),
                                };
                                if let Some(response) = response {
                                    if let Err(err) = link.send(Event::from(&dbg, response)) {
                                        log::warn!("{dbg}.run | Can't send reply: {:?}", err);
                                    }
                                }
                            }
                            Err(err) => {
                                log::trace!("{dbg}.run | parse error: {:?}", err)
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!("{}.run | TcpStream read error: {:?}", dbg, err);
                        if let IsConnected::Closed(_) = Self::parse_err(&dbg, err) {
                            exit.store(true, Ordering::Release);
                            if let Err(err) = Self::close(&dbg, &stream) {
                                log::warn!("{dbg}.run | Close tcp stream error: {:?}", err);
                            }
                            ctx.exit();
                        }
                    }
                }
            }
            log::debug!("{dbg}.run | Exit");
            Ok(())
        })?;
        self.handles.push(handle);
        Ok(())
    }
    ///
    /// Sends messages to the TCP Socket
    fn send(&self, stream: &TcpStream, hub: Arc<Hub>) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "send");
        let stream = stream.try_clone().map_err(|err| error.pass_with(format!("Can't clone stream"), err.to_string()))?;
        let exit = self.exit.clone();
        let handle = hub.listen::<Event<QueryId>, Option<()>>(self.scheduler.clone(), move |event: Event<QueryId>, _| {
            let mut w_stream = BufWriter::new(&stream);
            let mut message = Self::tcp_message(&dbg);
            // let bytes = message.build(&event.bytes, event.msg_id);
            // FieldConf::Const(vec![0x22]),   // Syn
            // FieldConf::Bytes,               // ID
            // FieldConf::Byte,                // Content
            // FieldConf::Byte,                // Cot
            // FieldConf::U32Be,               // QueryId
            // FieldConf::U32Be,               // Size
            // FieldConf::Bytes,               // Payload bytes
            let bytes = message.build(&[
                Field::Const,
                Field::U32(event.id),
                Field::Byte(event.content.into()),
                Field::Byte(event.cot as u8),
                Field::U32(event.query_id as u32),
                Field::U32(event.bytes.len() as u32),
                Field::Bytes(event.bytes),
            ]);
            if let Err(err) = w_stream.write_all(&bytes) {
                log::warn!("{dbg}.run | TcpStream write error: {:?}", err);
                if let Err(err) = Self::close(&dbg, &stream) {
                    log::warn!("{dbg}.run | Close tcp stream error: {:?}", err);
                }
                exit.store(true, Ordering::Release);
            }
            None
        })?;
        self.handles.push(handle);
        Ok(())
    }
    ///
    /// Returns Connection status dipending on IO Error
    fn parse_err(dbg: &Dbg, input: std::io::Error) -> IsConnected<(), Error> {
        // log::warn!("{}.parse_err | error reading from socket: {:?}", dbg, input);
        // log::warn!("{}.parse_err | error kind: {:?}", dbg, input.kind());
        let err = Error::new(dbg, "parse_err").pass(&input.to_string());
        match input.kind() {
            // std::io::ErrorKind::NotFound => todo!(),
            std::io::ErrorKind::PermissionDenied => IsConnected::Closed(err),
            std::io::ErrorKind::ConnectionRefused => IsConnected::Closed(err),
            std::io::ErrorKind::ConnectionReset => IsConnected::Closed(err),
            std::io::ErrorKind::HostUnreachable => IsConnected::Closed(err),
            std::io::ErrorKind::NetworkUnreachable => IsConnected::Closed(err),
            std::io::ErrorKind::ConnectionAborted => IsConnected::Closed(err),
            std::io::ErrorKind::NotConnected => IsConnected::Closed(err),
            std::io::ErrorKind::AddrInUse => IsConnected::Closed(err),
            std::io::ErrorKind::AddrNotAvailable => IsConnected::Closed(err),
            std::io::ErrorKind::NetworkDown => IsConnected::Closed(err),
            std::io::ErrorKind::BrokenPipe => IsConnected::Closed(err),
            std::io::ErrorKind::AlreadyExists => IsConnected::Closed(err),
            std::io::ErrorKind::WouldBlock => IsConnected::Active(()),
            // std::io::ErrorKind::NotADirectory => todo!(),
            // std::io::ErrorKind::IsADirectory => todo!(),
            // std::io::ErrorKind::DirectoryNotEmpty => todo!(),
            // std::io::ErrorKind::ReadOnlyFilesystem => todo!(),
            // std::io::ErrorKind::FilesystemLoop => todo!(),
            // std::io::ErrorKind::StaleNetworkFileHandle => todo!(),
            // std::io::ErrorKind::InvalidInput => todo!(),
            // std::io::ErrorKind::InvalidData => todo!(),
            std::io::ErrorKind::TimedOut => IsConnected::Active(()),
            // std::io::ErrorKind::WriteZero => todo!(),
            // std::io::ErrorKind::StorageFull => todo!(),
            // std::io::ErrorKind::NotSeekable => todo!(),
            // std::io::ErrorKind::FilesystemQuotaExceeded => todo!(),
            // std::io::ErrorKind::FileTooLarge => todo!(),
            // std::io::ErrorKind::ResourceBusy => todo!(),
            // std::io::ErrorKind::ExecutableFileBusy => todo!(),
            // std::io::ErrorKind::Deadlock => todo!(),
            // std::io::ErrorKind::CrossesDevices => todo!(),
            // std::io::ErrorKind::TooManyLinks => todo!(),
            // std::io::ErrorKind::InvalidFilename => todo!(),
            // std::io::ErrorKind::ArgumentListTooLong => todo!(),
            // std::io::ErrorKind::Interrupted => todo!(),
            // std::io::ErrorKind::Unsupported => todo!(),
            // std::io::ErrorKind::UnexpectedEof => todo!(),
            // std::io::ErrorKind::OutOfMemory => todo!(),
            // std::io::ErrorKind::Other => todo!(),
            _ => IsConnected::Closed(err),
        }
    }
    ///
    /// Closes a connection
    pub fn close(dbg: &Dbg, stream: &TcpStream) -> Result<(), Error> {
        stream
            .shutdown(Shutdown::Both)
            .map_err(|err| Error::new(dbg, "close").pass(err.to_string()))
    }
    ///
    /// Checks if the Service has finished running.
    /// 
    /// To force finish the Service call `exit`
    pub fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    ///
    /// Returns when internal thread's will finished
    #[allow(unused)]
    pub fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    ///
    /// Sends exit signal to main tread
    pub fn exit(&self) {
        self.exit.store(true, Ordering::Release);

    }
}
///
/// Connection status
enum IsConnected<T, E> {
    #[allow(unused)]
    Active(T),
    Closed(E),
}
