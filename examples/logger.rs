extern crate multican;
use multican::{CanBusType, CanConfig};

// to see something, try cansend vcan0 123#4570

fn main() {
    env_logger::init();
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
    ];
    let mut network = multican::from_config(cfg);
    loop {
        for message in network.recv() {
            println!("RX: {:?}", message);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
