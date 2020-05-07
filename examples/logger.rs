extern crate multican;
use multican::{CanConfig, CanBusType};

// to see something, try cansend can2 123#4570

fn main() {
    let cfg = vec![
        CanConfig {
            id: 0,
            kind: CanBusType::VirtualSocketCan,
        },
        CanConfig {
            id: 1,
            kind: CanBusType::SocketCan,
        },
        CanConfig {
            id: 2,
            kind: CanBusType::SocketCan,
        },
        // you can also mix network kinds
        CanConfig {
            id: 3,
            kind: CanBusType::Udp,
        },
    ];
    let mut network = multican::from_config(cfg);
    loop {
        for message in network.recv() {
            println!("RX: {:?}", message);
        }
    }
}
