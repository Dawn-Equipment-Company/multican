//use futures::stream::{Stream, StreamExt};
use multican::async_can_udp::AsyncUdpNetwork;
use multican::async_socketcan::AsyncSocketCanNetwork;
use multican::{CanBusType, CanConfig};
use std::error::Error;
use tokio::stream::StreamExt;
use tokio::sync::mpsc;

// lots of this code came from

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = vec![
        CanConfig {
            id: 0,
            kind: CanBusType::VirtualSocketCan,
        },
        CanConfig {
            id: 1,
            kind: CanBusType::SocketCan,
        },
    ];

    //let mut network = AsyncUdpNetwork::new(0);
    //let mut network = AsyncSocketCanNetwork::new(0, "vcan");
    //let mut n0 = AsyncSocketCanNetwork::new(0, "vcan");
    //let mut n1 = AsyncSocketCanNetwork::new(1, "vcan");

    // can't get this to compile, wants to copy which i can't implement
    //let rx = n0.clone().socket.merge(n1.socket);
    //let mut network = multican::from_config_async(cfg);

    // this works:
    /*let t0 = tokio::spawn(async move {
        loop {
        while let Some(next) = n0.socket.next().await {
            println!("RX: {:?}", next);
        }
        }
    });
    let t1 = tokio::spawn(async move {
        loop {
        while let Some(next) = n1.socket.next().await {
            println!("RX: {:?}", next);
        }
        }
    });
    tokio::join!(t0, t1);*/
    // end 'this works'
    // still want to merge the streams
    /*
    tokio::spawn(async move {
        loop {
        while let Some(next) = n1.socket.next().await {
            println!("RX: {:?}", next);
        }
        }
    });*/

    // this also works and i can possibly put it in multican
    // kind of weird though, i'm not sure how to make it better
    /*let (mut tx, mut rx) = mpsc::channel(10);
    let mut tx1 = tx.clone();
    let t0 = tokio::spawn(async move {
        loop {
        while let Some(next) = n0.socket.next().await {
            println!("RX0: {:?}", next);
            tx1.send(next).await.unwrap();
        }
        }
    });
    let mut tx2 = tx.clone();
    let t1 = tokio::spawn(async move {
        loop {
        while let Some(next) = n1.socket.next().await {
            println!("RX1: {:?}", next);
            tx2.send(next).await.unwrap();
        }
        }
    });

    while let Some(msg) = rx.next().await {
        println!("Merge rx: {:?}", msg);
    }*/

    let mut network = multican::from_config_async(cfg);
    let mut can_stream = network.stream();
    while let Some(msg) = can_stream.next().await {
        println!("RX: {:?}", msg);
    }

    //loop {
    // this doesn't really work since it waits until both receive a value to move through the
    // loop
    /*if let Ok(m) = n0.socket.try_next().await {
        println!("rx0: {:?}", m);
    }
    if let Ok(m) = n1.socket.try_next().await {
        println!("rx1: {:?}", m);
    }*/
    //    while let Some(next) = n0.socket.next().await {
    //        println!("RX: {:?}", next);
    //    }
    //}
    Ok(())
}
