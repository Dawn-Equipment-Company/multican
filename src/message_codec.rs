use crate::can_message::CanMessage;
use std::io;
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};

pub struct CanCodec;

impl CanCodec {
    pub fn new() -> Self {
        CanCodec { }
    }
}

impl Decoder for CanCodec {
    type Item = CanMessage;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            Ok(None)
        } else {
            let m = CanMessage::from_bytes(buf);
            buf.clear();
            Ok(Some(m))
        }
    }
}

impl Encoder<CanMessage> for CanCodec {
    //type Item = CanMessage;
    type Error = io::Error;

    fn encode(&mut self, item: CanMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        trace!("Encoding {:?}", item);
        let b = item.into_bytes();
        dst.put_slice(&b);
        Ok(())
    }
}
