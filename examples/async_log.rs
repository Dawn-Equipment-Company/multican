use futures::StreamExt;
use multican::{CanBusType, CanConfig, CanMessage};
use std::error::Error;
use tracing::{debug, info, Instrument};
use tracing_subscriber::prelude::*;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    // enable this to log to the console:
    //
    // console_subscriber::init();

    // enable these to send traces to a running jaeger server:
    //
    // let tracer = opentelemetry_jaeger::new_pipeline()
    //     .with_service_name("async_log")
    //     .install_simple()?;
    // let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    // tracing_subscriber::registry()
    //     .with(opentelemetry)
    //     .try_init()?;

    let root = tracing::span!(tracing::Level::TRACE, "lifecycle");
    let _enter = root.enter();

    let cfg = vec![CanConfig {
        id: 1,
        kind: CanBusType::Udp,
    }];

    let mut network = multican::from_config_async(cfg);
    let mut can_stream = network.stream().await;

    let sender = tokio::spawn(async move {
        let mut counter = 0;

        // while counter < 30 {

        loop {
            // tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            // println!("{} >>>>>>>>>>>>>>>>>>>>>>>>>>>>>", counter);

            network
                .send(CanMessage {
                    header: 5553,
                    data: vec![1, 2, 3, 4, 5],
                    bus: 1,
                })
                .await;

            // println!("{} <<<<<<<<<<<<<<<<<<<<<<<<<<<<<", counter);

            counter += 1;

            if counter % 1000 == 0 {
                println!("SENT: {}", counter);
            }

            if counter >= 100_000 {
                return;
            }
        }
    });

    let t = tokio::spawn(async move {
        let mut counter = 0;

        while let Some(msg) = can_stream.next().await {
            // debug!("XXXXX RECEIVED {:?}", msg);

            counter += 1;

            if counter % 1000 == 0 {
                println!("RECEIVED: {}", counter);
            }
        }
    });

    tokio::time::sleep(std::time::Duration::from_secs(300)).await;

    // let (tt, ss) = futures::join!(t, sender);
    // tt.unwrap();
    // ss.unwrap();

    Ok(())
}
