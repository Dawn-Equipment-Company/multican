use crate::can_message::CanMessage;
#[cfg(feature = "async-tokio")]
use async_trait::async_trait;

pub trait CanNetwork {
    fn send(&self, msg: CanMessage);
    fn recv(&self) -> Vec<CanMessage>;
}

#[cfg(feature = "async-tokio")]
#[async_trait]
pub trait AsyncCanNetwork {
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error>;
}
