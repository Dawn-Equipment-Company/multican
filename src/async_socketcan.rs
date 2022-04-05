use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{AsyncCanNetwork, CanMessage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use socketcan::CANFrame;
use tokio_socketcan::CANSocket;

/// SocketCAN adapter for mulitcan
pub struct AsyncSocketCanNetwork {
    pub socket: CANSocket,
    pub bus: u8,
}

#[async_trait]
impl AsyncCanNetwork for AsyncSocketCanNetwork {
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        trace!("Sending {:?}", msg);
        let frame = CANFrame::new(msg.header, &msg.data, false, false)
            .expect("failed to convert can message to frame");
        self.socket.write_frame(frame).unwrap();
        Ok(())
    }

    async fn next(&mut self) -> Option<CanMessage> {
        if let Some(Ok(frame)) = self.socket.next().await {
            Some(CanMessage {
                header: frame.id(),
                data: frame.data().to_owned(),
                bus: self.bus,
            })
        } else {
            None
        }
    }
}

impl Stream for AsyncSocketCanNetwork {
    type Item = CanMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        println!("poll_next");
        match self.socket.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                println!("got frame");
                Poll::Ready(Some(CanMessage {
                    header: frame.id(),
                    data: frame.data().to_vec(),
                    bus: self.bus,
                }))
            }
            Poll::Ready(Some(Err(_e))) => Poll::Ready(None),
            _ => Poll::Pending,
        }
    }
}

impl AsyncSocketCanNetwork {
    /// id is the network interface prefix, usually 'can' or 'vcan'
    pub fn new(bus: u8, id: &'static str) -> Self {
        debug!("Initializing bus #{}", bus);
        let bus_id = format!("{}{}", id, bus);
        debug!("Opening bus number {} - id: {}", bus, bus_id);
        let socket = CANSocket::open(&bus_id).expect("Failed to open bus");
        /*socket
            .set_read_timeout(time::Duration::from_millis(100))
            .expect("Failed to set socketcan read timeout");
        socket
            .set_nonblocking(true)
            .expect("Failed to set socketcan socket to nonblocking");*/

        debug!("opened {}", bus_id);
        //let s2 = CANSocket::open("can2").expect("failed to open can2");
        //let mut x = socket.merge(s2);
        //socket.split();
        AsyncSocketCanNetwork { socket, bus }
    }
}
