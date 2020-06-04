use crate::can_message::CanMessage;
use crate::can_network::CanNetwork;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct MultiCan {
    networks: HashMap<u8, Box<dyn CanNetwork + Send>>,
}

impl MultiCan {
    pub fn new() -> Self {
        MultiCan {
            networks: HashMap::new(),
        }
    }

    pub fn add_adapter(&mut self, id: u8, adapter: Box<dyn CanNetwork + Send>) {
        self.networks.insert(id, adapter);
    }

    /// Sends a single CAN message on the bus specified by the message
    pub fn send(&mut self, msg: CanMessage) {
        match self.networks.entry(msg.bus) {
            Entry::Occupied(n) => {
                n.into_mut().send(msg);
            }
            Entry::Vacant(_) => warn!("MultiCan: missing adapter for bus {}", msg.bus),
        };
    }

    /// Receives messages from any configured bus
    pub fn recv(&mut self) -> Vec<CanMessage> {
        let mut messages: Vec<CanMessage> = Vec::new();

        for (_key, value) in &mut self.networks {
            messages.append(&mut value.recv());
        }

        messages
    }
}
