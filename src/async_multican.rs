use crate::can_message::CanMessage;
use crate::can_network::AsyncCanNetwork;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AsyncMultiCan {
    networks: HashMap<u8, Arc<dyn AsyncCanNetwork + 'static>>,
}

impl<'a> AsyncMultiCan {
    pub fn new() -> Self {
        AsyncMultiCan {
            networks: HashMap::new(),
        }
    }

    pub fn add_adapter(&mut self, id: u8, adapter: Arc<dyn AsyncCanNetwork + 'static>) {
        self.networks.insert(id, adapter);
    }

    /// Sends a single CAN message on the bus specified by the message
    pub async fn send(&mut self, msg: CanMessage) {
        println!("here before send");
        if let Some(network) = self.networks.get_mut(&msg.bus) {
            println!("got network");
            println!("async multican send");
            trace!("TX: {:?}", msg);
            let network = network.clone();
            tokio::spawn(async move {
                network.send(msg).await.expect("unable to send");
            });
            println!("after network.send(msg)");
        } else {
            warn!("AsyncMultiCan: missing adapter for bus {}", msg.bus)
        }
    }

    // this works, but i don't know what bus the message came in on
    /*pub fn stream(&mut self) -> tokio::sync::mpsc::Receiver<CanMessage> {
        let (mut tx, rx) = mpsc::channel(10);
        let mut sa = futures::stream::SelectAll::new();
        for (_k, v) in self.networks.iter() {
            sa.push(v.socket.clone());
        }
        let _ = tokio::spawn(async move {
            while let Some(next) = sa.next().await {
                if let Ok(frame) = next {
                    println!("frame: {:?}", frame);
                    let msg = CanMessage {
                        header: frame.id(),
                        data: frame.data().to_vec(),
                        bus: 0,
                    };
                    tx.send(msg).await.unwrap();
                }
            }
        });
        rx
    }*/

    // this one gets the bus number correctly, but doesn't seem very efficient.  shouldn't have to
    // spawn a task for each bus since they're async, but oh well
    pub async fn stream(&mut self) -> tokio_stream::wrappers::ReceiverStream<CanMessage> {
        let (tx, rx) = mpsc::channel(10);

        // let networks = self.networks.clone();

        // tokio::spawn(async move {
        for network in self.networks.values() {
            let t = tx.clone();
            let network = network.clone();

            tokio::spawn(async move {
                println!("spawned listener");

                while let Some(next) = network.next().await {
                    println!("recv");
                    t.send(next).await.unwrap();
                }
            });
        }
        // });

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}
