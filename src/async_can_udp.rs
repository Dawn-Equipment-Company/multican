use crate::message_codec::CanCodec;
use crate::{AsyncCanNetwork, CanMessage};
use async_trait::async_trait;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::udp::UdpFramed;

// lots of this came from:
// https://github.com/henninglive/tokio-udp-multicast-chat/blob/master/src/main.rs
//
const DEFAULT_PORT: u16 = 25000;
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[derive(Debug)]
struct Send(CanMessage);

#[derive(Debug)]
struct Next(tokio::sync::oneshot::Sender<CanMessage>);

#[derive(Clone)]
pub struct AsyncUdpNetwork {
    send_tx: Sender<Send>,
    next_tx: Sender<Next>,
}

impl AsyncUdpNetwork {
    pub fn new(bus_number: u8) -> Self {
        //let multicast_group = Ipv4Addr::new(239, 0, 0, bus_number + 222);
        let multicast_group = [239, 0, 0, bus_number + 222];

        debug!("joining multicast group {:?}", multicast_group);

        let address = SocketAddrV4::new(IP_ALL.into(), DEFAULT_PORT);
        let multi_addr = SocketAddrV4::new(multicast_group.into(), DEFAULT_PORT);

        let std_socket = AsyncUdpNetwork::bind_multicast(&address, &multi_addr).unwrap();
        let socket = UdpSocket::from_std(std_socket).unwrap();

        let socket = UdpFramed::new(socket, CanCodec::new());
        let (socket_tx, socket_rx) = socket.split();

        let (send_tx, send_rx) = tokio::sync::mpsc::channel(32);
        let (next_tx, next_rx) = tokio::sync::mpsc::channel(32);

        let inner = AsyncUdpNetworkImpl {
            socket_tx,
            socket_rx,
            address: std::net::SocketAddr::V4(address),
            bus: bus_number,
        };

        handle_messages(inner, send_rx, next_rx);

        Self { send_tx, next_tx }
    }

    fn bind_multicast(
        addr: &SocketAddrV4,
        multi_addr: &SocketAddrV4,
    ) -> Result<std::net::UdpSocket, Box<dyn Error>> {
        use socket2::{Domain, Protocol, Socket, Type};

        assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

        socket.set_reuse_address(true)?;
        socket.bind(&socket2::SockAddr::from(*addr))?;
        socket.set_multicast_loop_v4(true)?;
        socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

        Ok(std::net::UdpSocket::from(socket))
    }
}

fn handle_messages(
    mut inner: AsyncUdpNetworkImpl,
    mut send_rx: Receiver<Send>,
    mut next_rx: Receiver<Next>,
) {
    tokio::spawn(async move {
        loop {
            println!("loop start");

            tokio::select! {
                Some(Send(m)) = send_rx.recv() => {
                    println!("SEND COMMAND received");
                    inner.send(m).await.unwrap()
                }
                Some(Next(reply_tx)) = next_rx.recv() => {
                    println!("received request for next");
                    if let Some(m) = inner.next().await {
                        println!("next inner replied");
                        reply_tx.send(m).unwrap();
                        println!("next replied");
                    } else  {
                        println!("next got NONe");
                    }
                }
                else => {
                    println!("else");
                }
            }
        }
    });
}

#[async_trait]
impl AsyncCanNetwork for AsyncUdpNetwork {
    async fn send(&self, msg: CanMessage) -> Result<(), std::io::Error> {
        let res = self
            .send_tx
            .send(Send(msg))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e));

        println!("managed to send to inner");

        res
    }

    async fn next(&self) -> Option<CanMessage> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

        self.next_tx
            .send(Next(reply_tx))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
            .unwrap();

        println!("sent Next(reply_tx) to inner");

        if let Ok(m) = reply_rx.await {
            println!("some");
            Some(m)
        } else {
            println!("NOne");
            None
        }
    }
}

struct AsyncUdpNetworkImpl {
    // pub socket: UdpFramed<CanCodec>,
    socket_tx: SplitSink<UdpFramed<CanCodec>, (CanMessage, SocketAddr)>,
    socket_rx: SplitStream<UdpFramed<CanCodec>>,
    address: SocketAddr,
    bus: u8,
}

// #[async_trait]
impl AsyncUdpNetworkImpl {
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        println!("SENDING {:?}", msg);
        self.socket_tx.send((msg, self.address)).await.unwrap();
        Ok(())
    }

    async fn next(&mut self) -> Option<CanMessage> {
        println!("inner next");

        if let Some(Ok((frame, _addr))) = self.socket_rx.next().await {
            println!("inner inner some");
            Some(CanMessage {
                header: frame.header,
                data: frame.data,
                bus: self.bus,
            })
        } else {
            println!("inner inner none");
            None
        }
    }
}

impl Drop for AsyncUdpNetworkImpl {
    fn drop(&mut self) {
        trace!("Closing UDP connection");
    }
}
