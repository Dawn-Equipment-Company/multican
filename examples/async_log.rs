use futures::StreamExt;
use multican::{CanBusType, CanConfig, CanMessage};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = vec![
        CanConfig {
            id: 1,
            kind: CanBusType::Udp,
        },
        // CanConfig {
        //     id: 3,
        //     kind: CanBusType::Udp,
        // },
        // CanConfig {
        //     id: 0,
        //     kind: CanBusType::VirtualSocketCan,
        // },
    ];

    let mut network = multican::from_config_async(cfg);
    let mut can_stream = network.stream().await;
    println!("can stream returned");

    let t = tokio::spawn(async move {
        println!("1");

        while let Some(msg) = can_stream.next().await {
            println!("2");
            println!("RECEIVED: {:?}", msg);
        }
    });

    let sender = tokio::spawn(async move {
        // let mut counter = 0;

        while true {
            // tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

            // println!("{} >>>>>>>>>>>>>>>>>>>>>>>>>>>>>", counter);

            network
                .send(CanMessage {
                    header: 5553,
                    data: vec![1, 2, 3, 4, 5],
                    bus: 1,
                })
                .await;

            // println!("{} <<<<<<<<<<<<<<<<<<<<<<<<<<<<<", counter);

            // counter += 1;
        }
    });

    tokio::time::sleep(std::time::Duration::from_secs(300)).await;

    // let (tt, ss) = futures::join!(t, sender);
    // tt.unwrap();
    // ss.unwrap();

    Ok(())
}
