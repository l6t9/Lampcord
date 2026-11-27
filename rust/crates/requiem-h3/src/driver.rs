use std::future;
use std::io::ErrorKind;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Instant as TimeInstant;
use tokio::time;

use bytes::BytesMut;
use quinn_proto::{
    ClientConfig, Connection, ConnectionHandle, DatagramEvent, Endpoint, EndpointConfig, Event,
    TransportConfig,
};
use tokio::net::UdpSocket;

use crate::crypto;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Connect(#[from] quinn_proto::ConnectError),
    #[error("socket: {0}")]
    Io(#[from] io::Error),
    #[error("connection lost: {0}")]
    Lost(quinn_proto::ConnectionError),
    #[error("{0}")]
    Other(String),
}

pub struct Driver {
    socket: UdpSocket,
    remote: SocketAddr,
    endpoint: Endpoint,
    handle: ConnectionHandle,
    pub conn: Connection,
    recv_buf: Vec<u8>,
    scratch: Vec<u8>,
}

impl Driver {
    pub async fn connect(
        remote: SocketAddr,
        server_name: &str,
        crypto: Arc<crypto::ClientConfig>,
        transport: TransportConfig,
        cid_len: usize,
    ) -> Result<Self, Error> {
        let bind: SocketAddr = if remote.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()
        .unwrap();
        let socket = UdpSocket::bind(bind).await?;
        let mut endpoint_config = EndpointConfig::default();
        endpoint_config.cid_generator(move || {
            Box::new(quinn_proto::RandomConnectionIdGenerator::new(cid_len))
        });
        let mut endpoint = Endpoint::new(Arc::new(endpoint_config), None, false, None);
        let mut config = ClientConfig::new(crypto);
        config.transport_config(Arc::new(transport));

        config.initial_dst_cid_provider(Arc::new(|| {
            let mut id = [0u8; 8];
            rand::fill(&mut id);
            quinn_proto::ConnectionId::new(&id)
        }));
        let (handle, conn) = endpoint.connect(Instant::now(), config, remote, server_name)?;
        let mut driver = Self {
            socket,
            remote,
            endpoint,
            handle,
            conn,
            recv_buf: vec![0; 65536],
            scratch: Vec::new(),
        };
        driver.flush().await?;
        Ok(driver)
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        loop {
            while let Some(event) = self.conn.poll_endpoint_events() {
                if let Some(back) = self.endpoint.handle_event(self.handle, event) {
                    self.conn.handle_event(back);
                }
            }
            self.scratch.clear();
            match self
                .conn
                .poll_transmit(Instant::now(), 1, &mut self.scratch)
            {
                Some(transmit) => {
                    self.socket
                        .send_to(&self.scratch[..transmit.size], transmit.destination)
                        .await?;
                }
                None => return Ok(()),
            }
        }
    }

    pub async fn ready(&mut self) -> Result<(), Error> {
        let timeout = self.conn.poll_timeout();
        tokio::select! {
            r = self.socket.readable() => r.map_err(Error::Io),
            _ = async {
                match timeout {
                    Some(at) => time::sleep_until(TimeInstant::from_std(at)).await,
                    None => future::pending().await,
                }
            } => Ok(()),
        }
    }

    pub fn process(&mut self) -> Result<Vec<Event>, Error> {
        loop {
            match self.socket.try_recv_from(&mut self.recv_buf) {
                Ok((len, from)) => {
                    let mut response = Vec::new();
                    if let Some(event) = self.endpoint.handle(
                        Instant::now(),
                        from,
                        None,
                        None,
                        BytesMut::from(&self.recv_buf[..len]),
                        &mut response,
                    ) {
                        match event {
                            DatagramEvent::ConnectionEvent(handle, event)
                                if handle == self.handle =>
                            {
                                self.conn.handle_event(event)
                            }
                            DatagramEvent::Response(transmit) => {
                                let _ = self
                                    .socket
                                    .try_send_to(&response[..transmit.size], self.remote);
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,

                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(e) => return Err(Error::Io(e)),
            }
        }
        if self
            .conn
            .poll_timeout()
            .is_some_and(|at| at <= Instant::now())
        {
            self.conn.handle_timeout(Instant::now());
        }
        let mut events = Vec::new();
        while let Some(event) = self.conn.poll() {
            events.push(event);
        }
        Ok(events)
    }

    pub async fn step(&mut self) -> Result<Vec<Event>, Error> {
        self.ready().await?;
        let events = self.process()?;
        self.flush().await?;
        Ok(events)
    }
}
