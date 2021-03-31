use crate::async_socketcan::AsyncSocketCanNetwork;
use crate::can_message::CanMessage;
use crate::can_network::AsyncCanNetwork;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use futures::StreamExt;
//use tokio::stream::StreamExt;
use tokio::sync::mpsc;

pub struct AsyncMultiCan {
    //networks: HashMap<u8, Box<dyn AsyncCanNetwork + Send + 'static>>,
    networks: HashMap<u8, AsyncSocketCanNetwork>,
    //networks: HashMap<u8, Arc<Mutex<AsyncSocketCanNetwork>>>,
    //networks: HashMap<u8, Arc<Mutex<AsyncSocketCanNetwork>>>,
}

impl<'a> AsyncMultiCan {
    pub fn new() -> Self {
        AsyncMultiCan {
            networks: HashMap::new(),
        }
    }

    //pub fn add_adapter(&mut self, id: u8, adapter: Box<dyn AsyncCanNetwork + Send + 'static>) {
    pub fn add_adapter(&mut self, id: u8, adapter: AsyncSocketCanNetwork) {
        //jself.networks.insert(id, Arc::new(Mutex::new(adapter)));
        self.networks.insert(id, adapter);
    }

    /// Sends a single CAN message on the bus specified by the message
    pub async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        match self.networks.entry(msg.bus) {
            Entry::Occupied(n) => {
                trace!("TX: {:?}", msg);
                n.into_mut().send(msg).await
                //n.into_mut().lock().unwrap().send(msg).await
            }
            Entry::Vacant(_) => {
                warn!("AsyncMultiCan: missing adapter for bus {}", msg.bus);
                Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "You tried to use an unconfigured bus id"))
            }
        }
    }

    // this one gets the bus number correctly, but doesn't seem very efficient.  shouldn't have to
    // spawn a task for each bus since they're async, but oh well
    pub fn stream(&mut self) -> tokio::sync::mpsc::Receiver<CanMessage> {
        let (tx, rx) = mpsc::channel(10);
        /*tokio::spawn(async move {
        tokio::select! {
            m = self.networks.get_mut(&1).unwrap().socket.next() => {
                println!("m: {:?}", m);
            }
        }
        });*/
        for (k, v) in self.networks.iter_mut() {
            let mut s = v.socket.clone();
            let num = k.clone();
            let t = tx.clone();
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
        //tokio::spawn(async move {
        /*let n1 = self.networks.get(&1).unwrap().clone();
        tokio::spawn(async move  {
            let n1 = n1.lock().unwrap();
            println!("rx bus {}", n1.bus);
            let s = n1.socket.lock().unwrap();
            while let Some(test) = s.next().await {
                println!("test");
            }*/
            /*let n = n1.lock().unwrap();
            while let Some(text) = n.socket.next().await {
                println!("a");
            }*/
            //let s = n1 as AsyncSocketCanNetwork;
            //let r = n.socket.next().await;
            //println!("{:?}", r);
        //    println!("x");
        //});
        rx
    }

    /*pub fn stream2(&mut self) -> tokio::sync::mpsc::UnboundedReceiver<CanMessage> {
        // foreach network,
        //     spawn a listener task
        //     pass in the tx
        //  return the rx
        //  -- spawning the listener task doesn't work here because it wants to clone the socket
        //  which doesn't work for udp for some reason
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
