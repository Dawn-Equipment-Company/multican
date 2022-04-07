use crate::can_message::CanMessage;
#[cfg(feature = "async-tokio")]
use async_trait::async_trait;

pub trait CanNetwork {
    /// Queues a message to be sent on the bus
    fn send(&self, msg: CanMessage);
    /// Gets all messages currently available in the incoming message queue
    fn recv(&self) -> Vec<CanMessage>;
}

#[cfg(feature = "async-tokio")]
#[async_trait]
pub trait AsyncCanNetwork: Send + Sync {
    async fn send(&self, msg: CanMessage) -> Result<(), std::io::Error>;
    async fn next(&self) -> Option<CanMessage>;
    // fn stream(&self) -> futures::Stream<Item = CanMessage>;
}
