//extern crate socket2;

use crate::{CanMessage, CanNetwork};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub struct UdpNetwork {
    socket: UdpSocket,
    address: SocketAddr,
}

impl CanNetwork for UdpNetwork {
    fn send(&self, msg: CanMessage) {
        trace!("Sending {:?}", msg);
        let sent = self
            .socket
            .send_to(&msg.into_bytes(), &self.address)
            .unwrap();
        trace!("Sent {} bytes", sent);
    }

    fn recv(&self) -> Option<CanMessage> {
        let mut buf = [0u8; 64]; // receive buffer

        match self.socket.recv_from(&mut buf) {
            Ok((len, _remote_addr)) => {
                let data = &buf[..len];
                let msg = CanMessage::from_bytes(data);
                trace!("RX {:?}", msg);
                Some(msg)
            }
            Err(_) => None,
        }
    }
}

impl Drop for UdpNetwork {
    fn drop(&mut self) {
        println!("Closing UDP connection");
    }
}

impl UdpNetwork {
    pub fn new(bus_number: u8) -> Self {
        let multicast_group = Ipv4Addr::new(239, 0, 0, bus_number + 222);

        debug!("joining multicast group {:?}", multicast_group);

        let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket
            .set_nonblocking(true)
            .expect("Failed to set nonblocking socket");
        socket
            .set_reuse_address(true)
            .expect("Failed to set reuse address to true");
        #[cfg(unix)]
        socket
            .set_reuse_port(true)
            .expect("Failed to set SO_REUSEPORT");
        // if this is set, i never get messages from other applications on the same machine
        //socket.set_multicast_loop_v4(false).expect("Failed to set loop to false");
        socket
            .join_multicast_v4(&multicast_group, &Ipv4Addr::new(0, 0, 0, 0))
            .unwrap();

        socket
            .bind(&"0.0.0.0:25000".parse::<SocketAddr>().unwrap().into())
            .unwrap();
        let socket = socket.into_udp_socket();

        let address = SocketAddr::new(IpAddr::V4(multicast_group), 25000);
        UdpNetwork { socket, address }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    fn it_works() {
        let mut network = UdpNetwork::new(0);
        network.send(CanMessage {
            bus: 0,
            header: 0x12345678,
            data: vec![1, 2, 3],
        });
    }
}
*/
