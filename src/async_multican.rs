use crate::can_message::CanMessage;
use crate::can_network::AsyncCanNetwork;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

#[derive(Debug)]
pub struct AsyncMultiCan {
    networks: HashMap<u8, Arc<dyn AsyncCanNetwork + 'static>>,
}

impl<'a> AsyncMultiCan {
    #[tracing::instrument]
    pub fn new() -> Self {
        AsyncMultiCan {
            networks: HashMap::new(),
        }
    }

    #[tracing::instrument(skip(self, id, adapter))]
    pub fn add_adapter(&mut self, id: u8, adapter: Arc<dyn AsyncCanNetwork + 'static>) {
        self.networks.insert(id, adapter);
    }

    /// Sends a single CAN message on the bus specified by the message
    #[tracing::instrument(skip(self))]
    pub async fn send(&mut self, msg: CanMessage) {
        if let Some(network) = self.networks.get_mut(&msg.bus) {
            let network = network.clone();

            tokio::spawn(async move { network.send(msg).await.unwrap() })
                .await
                .expect("unable to send");
        } else {
            warn!("AsyncMultiCan: missing adapter for bus {}", msg.bus)
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn stream(&mut self) -> tokio_stream::wrappers::ReceiverStream<CanMessage> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        for network in self.networks.values().cloned() {
            let tx = tx.clone();

            tokio::spawn(async move {
                loop {
                    while let Some(next) = network.next().await {
                        let tx = tx.clone();

                        tokio::spawn(async move {
                            tx.send(next).await.unwrap();
                        });
                    }
                }
            });
        }

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}
