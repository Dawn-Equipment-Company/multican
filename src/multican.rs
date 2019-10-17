use crate::can_message::CanMessage;
use crate::can_network::CanNetwork;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct MultiCan {
    networks: HashMap<u8, Box<CanNetwork>>,
}

impl MultiCan {
    pub fn new(adapters: &mut Vec<Box<CanNetwork>>) -> Self {
        let mut networks = HashMap::new();
        // TODO: use config id number as the index
        // pop puts them in the wrong order, so reverse it
        networks.insert(2, adapters.pop().unwrap());
        networks.insert(1, adapters.pop().unwrap());
        networks.insert(0, adapters.pop().unwrap());
        // borrow problem here i don't feel like solving.  will want to fix
        // when we have more than 2 busses
        /*let mut idx = 0;
        for a in adapters {
            networks.insert(idx, adapters.pop().unwrap());
            idx += 1;
        }*/
        MultiCan { networks }
    }

    pub fn send(&mut self, msg: CanMessage) {
        match self.networks.entry(msg.bus) {
            Entry::Occupied(n) => {
                n.into_mut().send(msg);
            }
            Entry::Vacant(_) => println!("empty entry for {}", msg.bus),
        };
    }

    pub fn recv(&mut self) -> Vec<CanMessage> {
        let mut messages: Vec<CanMessage> = Vec::new();

        for (key, value) in &mut self.networks {
            if let Some(mut m) = value.recv() {
                m.bus = *key;
                messages.push(m);
            }
        }

        messages
    }
}
