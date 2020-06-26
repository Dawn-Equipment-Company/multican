//use futures::stream::{Stream, StreamExt};
use multican::async_can_udp::AsyncUdpNetwork;
use multican::async_socketcan::AsyncSocketCanNetwork;
use multican::{CanBusType, CanConfig};
use std::error::Error;
use tokio::stream::StreamExt;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = vec![
        CanConfig {
            id: 0,
            kind: CanBusType::VirtualSocketCan,
        },
        CanConfig {
            id: 1,
            kind: CanBusType::VirtualSocketCan,
        },
    ];

    let mut network = multican::from_config_async(cfg);
    let mut can_stream = network.stream_bad();
    while let Some(msg) = can_stream.next().await {
        println!("RX: {:?}", msg);
    }

    Ok(())
}
