use tokio::net::UdpSocket;
use tokio::stream::StreamExt;
use tokio::{io, time};
use tokio_util::codec::BytesCodec;
use tokio_util::udp::UdpFramed;
use futures::{FutureExt, SinkExt};
use std::net::SocketAddrV4;

use bytes::Bytes;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;
use multican::message_codec::CanCodec;

// lots of this code came from
// https://github.com/henninglive/tokio-udp-multicast-chat/blob/master/src/main.rs

const DEFAULT_PORT: u16 = 25000;
const DEFAULT_MULTICAST: [u8; 4] = [239, 0, 0, 222];
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let addr = SocketAddrV4::new(IP_ALL.into(), DEFAULT_PORT);
    let multi_addr = SocketAddrV4::new(DEFAULT_MULTICAST.into(), DEFAULT_PORT);

    let std_socket = bind_multicast(&addr, &multi_addr).unwrap();
    let a = UdpSocket::from_std(std_socket).unwrap();

    println!("waiting for incoming messages");
    let mut a = UdpFramed::new(a, CanCodec::new());

    loop {
        println!("waiting");
        let (msg, addr) = a.next().map(|e| e.unwrap()).await?;
        println!("RX: {:?}", msg);
    }


    Ok(())
}

fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
    ) -> Result<std::net::UdpSocket, Box<dyn Error>> {
    use socket2::{Domain, Type, Protocol, Socket};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let socket = Socket::new(
        Domain::ipv4(),
        Type::dgram(),
        Some(Protocol::udp()),
        )?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(
        multi_addr.ip(),
        addr.ip(),
        )?;

    Ok(socket.into_udp_socket())
}
