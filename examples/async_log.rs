use multican::{CanBusType, CanConfig};
use std::error::Error;

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
    // let mut can_stream = network.stream_bad();
    let mut can_stream = network.stream().await;
    while let Some(msg) = can_stream.recv().await {
        println!("RX: {:?}", msg);
    }

    Ok(())
}
