use crate::async_socketcan::AsyncSocketCanNetwork;
use crate::can_message::CanMessage;
use crate::can_network::AsyncCanNetwork;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use tokio::stream::StreamExt;
use tokio::sync::mpsc;

pub struct AsyncMultiCan {
    //networks: HashMap<u8, Box<dyn AsyncCanNetwork + Send + 'static>>,
    networks: HashMap<u8, AsyncSocketCanNetwork>,
}

impl<'a> AsyncMultiCan {
    pub fn new() -> Self {
        AsyncMultiCan {
            networks: HashMap::new(),
        }
    }

    //pub fn add_adapter(&mut self, id: u8, adapter: Box<dyn AsyncCanNetwork + Send + 'static>) {
    pub fn add_adapter(&mut self, id: u8, adapter: AsyncSocketCanNetwork) {
        self.networks.insert(id, adapter);
    }

    /// Sends a single CAN message on the bus specified by the message
    pub async fn send(&mut self, msg: CanMessage) {
        match self.networks.entry(msg.bus) {
            Entry::Occupied(n) => {
                trace!("TX: {:?}", msg);
                n.into_mut().send(msg).await.expect("Failed to send message");
            }
            Entry::Vacant(_) => warn!("AsyncMultiCan: missing adapter for bus {}", msg.bus),
        };
    }

    // this one gets the bus number correctly, but doesn't seem very efficient.  shouldn't have to
    // spawn a task for each bus since they're async, but oh well
    pub fn stream(&mut self) -> tokio::sync::mpsc::Receiver<CanMessage> {
        let (tx, rx) = mpsc::channel(10);
        for (k, v) in self.networks.iter_mut() {
            let mut s = v.socket.clone();
            let num = k.clone();
            let mut t = tx.clone();
            let _ = tokio::spawn(async move {
                while let Some(next) = s.next().await {
                    if let Ok(frame) = next {
                        let msg = CanMessage {
                            header: frame.id(),
                            data: frame.data().to_vec(),
                            bus: num,
                        };
                        t.send(msg).await.unwrap();
                    }
                }
            });
        }
        rx
    }

    /*pub fn stream2(&mut self) -> tokio::sync::mpsc::UnboundedReceiver<CanMessage> {
        // foreach network,
        //     spawn a listener task
        //     pass in the tx
        //  return the rx
        //  -- spawning the listener task doesn't work here because it wants to clone the socket
        //  which doesn't work for udb for some reason
        //  and i can't spawn inside listener because of lifetime issues i don't understand
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        for (k, v) in self.networks.iter() {
            let can_tx = tx.clone();
            let n = v.clone();
            let _ = tokio::spawn(async move {
                n.listen(can_tx);
            });
        }
        rx
    }*/
}
