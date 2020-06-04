use crate::can_message::CanMessage;
use tokio::stream::StreamExt;
use crate::can_network::AsyncCanNetwork;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use crate::async_socketcan::AsyncSocketCanNetwork;
use tokio::sync::mpsc;

pub struct AsyncMultiCan {
    networks: HashMap<u8, Box<dyn AsyncCanNetwork + Send>>,
}

impl AsyncMultiCan {
    pub fn new() -> Self {
        AsyncMultiCan {
            networks: HashMap::new(),
        }
    }

    pub fn add_adapter(&mut self, id: u8, adapter: Box<dyn AsyncCanNetwork + Send>) {
        self.networks.insert(id, adapter);
    }

    /// Sends a single CAN message on the bus specified by the message
    pub fn send(&mut self, msg: CanMessage) {
        match self.networks.entry(msg.bus) {
            Entry::Occupied(n) => {
                n.into_mut().send(msg);
            }
            Entry::Vacant(_) => warn!("AsyncMultiCan: missing adapter for bus {}", msg.bus),
        };
    }

    pub fn stream(&mut self) -> tokio::sync::mpsc::Receiver<CanMessage> {
        println!("stream called");
    let mut n0 = AsyncSocketCanNetwork::new(0, "vcan");
    let mut n1 = AsyncSocketCanNetwork::new(1, "can");
    let mut n2 = AsyncSocketCanNetwork::new(2, "can");
    let (mut tx, mut rx) = mpsc::channel(10);
    let mut tx1 = tx.clone();
    let t0 = tokio::spawn(async move {
        println!("Starting vcan0 listener");
        loop {
        while let Some(next) = n0.socket.next().await {
            if let Ok(frame) = next {
            //println!("RX0: {:?}", next);
                let msg = CanMessage {
                    header: frame.id(),
                    data: frame.data().to_vec(),
                    bus: 0,
                };
            tx1.send(msg).await.unwrap();
            }
        }
        }
    });
    let mut tx2 = tx.clone();
    let t1 = tokio::spawn(async move {
        println!("Starting can1 listener");
        loop {
        while let Some(next) = n1.socket.next().await {
            //println!("RX1: {:?}", next);
            if let Ok(frame) = next {
                let msg = CanMessage {
                    header: frame.id(),
                    data: frame.data().to_vec(),
                    bus: 1,
                };
            tx2.send(msg).await.unwrap();
            }
        }
        }
    });
    /*let mut tx3 = tx.clone();
    let t2 = tokio::spawn(async move {
        println!("Starting can2 listener");
        loop {
        while let Some(next) = n2.socket.next().await {
            //println!("RX1: {:?}", next);
            if let Ok(frame) = next {
                let msg = CanMessage {
                    header: frame.id(),
                    data: frame.data().to_vec(),
                    bus: 1,
                };
            tx3.send(msg).await.unwrap();
            }
        }
        }
    });*/
    rx

    /*while let Some(msg) = rx.next().await {
        println!("Merge rx: {:?}", msg);
    }*/
    }
    /*pub fn recv(&mut self) {
        let mut rx = self.networks[&0].socket.merge(self.networks[&1].socket);
    }*/
    // Receives messages from any configured bus
    /*pub fn recv(&mut self) -> Vec<CanMessage> {
        let mut messages: Vec<CanMessage> = Vec::new();

        for (key, value) in &mut self.networks {
            if let Some(mut m) = value.recv() {
                m.bus = *key;
                messages.push(m);
            }
        }

        messages
    }*/
}
