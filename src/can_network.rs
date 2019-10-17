use crate::can_message::CanMessage;

pub trait CanNetwork {
    fn send(&self, msg: CanMessage);
    fn recv(&self) -> Option<CanMessage>;
}
