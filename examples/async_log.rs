use tokio::stream::StreamExt;
use futures::FutureExt;
use std::error::Error;
use multican::async_can_udp::AsyncUdpNetwork;

// lots of this code came from

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut network = AsyncUdpNetwork::new(0);

    loop {
        let (msg, _addr) = network.socket.next().map(|e| e.unwrap()).await?;
        println!("RX: {:?}", msg);
    }
}

