use crate::message_codec::CanCodec;
use crate::{AsyncCanNetwork, CanMessage};
use async_trait::async_trait;
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

// lots of this came from:
// https://github.com/henninglive/tokio-udp-multicast-chat/blob/master/src/main.rs
//
const DEFAULT_PORT: u16 = 25000;
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

pub struct AsyncUdpNetwork {
    pub socket: UdpFramed<CanCodec>,
    address: SocketAddr,
}

#[async_trait]
impl AsyncCanNetwork for AsyncUdpNetwork {
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        trace!("Sending {:?}", msg);
        self.socket.send((msg, self.address)).await
    }

    /*async fn next(&self) -> futures::stream::Next<'_, CanMessage> {
        self.socket.next()
    }*/
    /*async fn listen(&mut self, tx: tokio::sync::mpsc::UnboundedSender<CanMessage>) {
//        tokio::spawn(async move {
            println!("listening");
            loop {
                while let Some(next) = self.socket.next().await {
                    if let Ok(frame) = next {
                        let msg = CanMessage {
                            header: 0x1234,
                            data: vec![1, 2, 3, 4],
                            bus: self.bus_number,
                        };
                        tx.send(msg);
                    }
                }

            }
//        });
    }*/
}

impl Drop for AsyncUdpNetwork {
    fn drop(&mut self) {
        trace!("Closing UDP connection");
    }
}

impl AsyncUdpNetwork {
    pub fn new(bus_number: u8) -> Self {
        todo!("not implemented");
        //let multicast_group = Ipv4Addr::new(239, 0, 0, bus_number + 222);
        /*let multicast_group = [239, 0, 0, bus_number + 222];

        debug!("joining multicast group {:?}", multicast_group);

        let address = SocketAddrV4::new(IP_ALL.into(), DEFAULT_PORT);
        let multi_addr = SocketAddrV4::new(multicast_group.into(), DEFAULT_PORT);

        let std_socket = AsyncUdpNetwork::bind_multicast(&address, &multi_addr).unwrap();
        let socket = UdpSocket::from_std(std_socket).unwrap();

        let socket = UdpFramed::new(socket, CanCodec::new());

        AsyncUdpNetwork {
            socket,
            address: std::net::SocketAddr::V4(address),
        }*/
    }

    fn bind_multicast(
        addr: &SocketAddrV4,
        multi_addr: &SocketAddrV4,
    ) -> Result<std::net::UdpSocket, Box<dyn Error>> {
        use socket2::{Domain, Protocol, Socket, Type};

        assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

        let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

        socket.set_reuse_address(true)?;
        socket.bind(&socket2::SockAddr::from(*addr))?;
        socket.set_multicast_loop_v4(true)?;
        socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

        Ok(socket.into_udp_socket())
    }
}
