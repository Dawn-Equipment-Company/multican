use crate::message_codec::CanCodec;
use crate::{AsyncCanNetwork, CanMessage};
use async_trait::async_trait;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio_util::udp::UdpFramed;
use tracing::{debug, error};

// lots of this came from:
// https://github.com/henninglive/tokio-udp-multicast-chat/blob/master/src/main.rs
//
const DEFAULT_PORT: u16 = 25000;
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[derive(Debug)]
struct Send(CanMessage);

#[derive(Debug)]
struct Next(tokio::sync::oneshot::Sender<CanMessage>);

#[derive(Clone, Debug)]
pub struct AsyncUdpNetwork {
    send_tx: Sender<Send>,
    next_tx: Sender<Next>,
}

impl AsyncUdpNetwork {
    #[tracing::instrument]
    pub fn new(bus_number: u8) -> Self {
        //let multicast_group = Ipv4Addr::new(239, 0, 0, bus_number + 222);

        let (send_tx, send_rx) = tokio::sync::mpsc::channel(32);
        let (next_tx, next_rx) = tokio::sync::mpsc::channel(32);

        let multicast_group = [239, 0, 0, bus_number + 222];

        debug!("joining multicast group {:?}", multicast_group);

        let address = SocketAddrV4::new(IP_ALL.into(), DEFAULT_PORT);
        let multi_addr = SocketAddrV4::new(multicast_group.into(), DEFAULT_PORT);

        let std_socket: std::net::UdpSocket =
            AsyncUdpNetwork::bind_multicast(&address, &multi_addr).unwrap();

        let socket: tokio::net::UdpSocket = UdpSocket::from_std(std_socket).unwrap();

        let socket = UdpFramed::new(socket, CanCodec::new());
        let (socket_tx, socket_rx) = socket.split();

        // COMMAND CHANNELS

        let inner = AsyncUdpNetworkImpl {
            socket_tx: Arc::new(Mutex::new(socket_tx)),
            socket_rx: Arc::new(Mutex::new(socket_rx)),
            address: std::net::SocketAddr::V4(address),
            bus: bus_number,
        };

        handle_messages(inner, send_rx, next_rx);

        Self { send_tx, next_tx }
    }

    #[tracing::instrument]
    fn bind_multicast(
        addr: &SocketAddrV4,
        multi_addr: &SocketAddrV4,
    ) -> Result<std::net::UdpSocket, Box<dyn Error>> {
        use socket2::{Domain, Protocol, Socket, Type};

        assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

        socket.bind(&socket2::SockAddr::from(*addr))?;
        // socket.set_nonblocking(true)?;
        socket.set_multicast_loop_v4(true)?;
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;
        socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

        Ok(std::net::UdpSocket::from(socket))
    }
}

#[tracing::instrument(skip(inner))]
fn handle_messages(
    mut inner: AsyncUdpNetworkImpl,
    mut send_rx: Receiver<Send>, // sending to a socket
    mut next_rx: Receiver<Next>, // nexting from a socket
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(Send(message)) = send_rx.recv() => {
                    inner.send(message).await.unwrap()
                }
                Some(Next(reply_tx)) = next_rx.recv() => {
                    inner.next(reply_tx).await;
                }
            }
        }
    });
}

#[async_trait]
impl AsyncCanNetwork for AsyncUdpNetwork {
    #[tracing::instrument]
    async fn send(&self, msg: CanMessage) -> Result<(), std::io::Error> {
        self.send_tx
            .send(Send(msg))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
    }

    #[tracing::instrument(skip(self))]
    async fn next(&self) -> Option<CanMessage> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

        let next_tx = self.next_tx.clone();

        tokio::spawn(async move {
            next_tx
                .send(Next(reply_tx))
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
                .unwrap();
        });

        match tokio::spawn(reply_rx).await.unwrap() {
            Ok(m) => Some(m),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }
}

#[derive(Debug)]
struct AsyncUdpNetworkImpl {
    socket_tx: Arc<Mutex<SplitSink<UdpFramed<CanCodec>, (CanMessage, SocketAddr)>>>,
    socket_rx: Arc<Mutex<SplitStream<UdpFramed<CanCodec>>>>,
    address: SocketAddr,
    bus: u8,
}

impl AsyncUdpNetworkImpl {
    #[tracing::instrument(skip(self))]
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        let socket_tx = self.socket_tx.clone();
        let address = self.address;

        tokio::spawn(async move {
            let mut socket_tx = socket_tx.lock().await;
            socket_tx.send((msg.clone(), address)).await.unwrap();
        });

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn next(&mut self, reply_tx: tokio::sync::oneshot::Sender<CanMessage>) {
        let socket_rx = self.socket_rx.clone();
        let bus = self.bus;

        tokio::spawn(async move {
            let mut socket_rx = socket_rx.lock().await;

            if let Some(Ok((frame, _addr))) = socket_rx.next().await {
                reply_tx
                    .send(CanMessage {
                        header: frame.header,
                        data: frame.data,
                        bus,
                    })
                    .unwrap();
            };
        });
    }
}
