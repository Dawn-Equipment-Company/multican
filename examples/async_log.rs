use futures::StreamExt;
use multican::{CanBusType, CanConfig};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = vec![
        CanConfig {
            id: 0,
            kind: CanBusType::Udp,
        },
        CanConfig {
            id: 1,
            kind: CanBusType::Udp,
        },
    ];

    let mut network = multican::from_config_async(cfg);
    let mut can_stream = network.stream().await;

    while let Some(msg) = can_stream.next().await {
        println!("RX: {:?}", msg);
    }

    Ok(())
}
