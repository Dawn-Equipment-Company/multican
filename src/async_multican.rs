use crate::can_message::CanMessage;
use crate::can_network::AsyncCanNetwork;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AsyncMultiCan {
    networks: HashMap<u8, Arc<Mutex<dyn AsyncCanNetwork + 'static>>>,
    // networks: HashMap<u8, AsyncSocketCanNetwork>,
}

impl<'a> AsyncMultiCan {
    pub fn new() -> Self {
        AsyncMultiCan {
            networks: HashMap::new(),
        }
    }

    pub fn add_adapter(&mut self, id: u8, adapter: Arc<Mutex<dyn AsyncCanNetwork + 'static>>) {
        self.networks.insert(id, adapter);
    }

    /// Sends a single CAN message on the bus specified by the message
    pub async fn send(&mut self, msg: CanMessage) {
        if let Some(network) = self.networks.get_mut(&msg.bus) {
            trace!("TX: {:?}", msg);
            network
                .lock()
                .await
                .send(msg)
                .await
                .expect("unable to send");
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
    pub async fn stream(&mut self) -> tokio::sync::mpsc::Receiver<CanMessage> {
        let (tx, rx) = mpsc::channel(10);

        for network in self.networks.values_mut() {
            let t = tx.clone();
            let network = network.clone();

            tokio::spawn(async move {
                while let Some(next) = network.lock().await.next().await {
                    t.send(next).await.unwrap();
                }
            });
        }

        rx
    }
}
