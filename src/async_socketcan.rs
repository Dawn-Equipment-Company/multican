use crate::{AsyncCanNetwork, CanMessage};
use socketcan::CANFrame;
//use tokio::stream::StreamExt;
//use crate::tokio_socketcan::CANSocket;
use tokio_socketcan::CANSocket;
use async_trait::async_trait;

/// SocketCAN adapter for mulitcan
pub struct AsyncSocketCanNetwork {
    pub socket: CANSocket,
    pub bus: u8,
}

#[async_trait]
impl AsyncCanNetwork for AsyncSocketCanNetwork {
    //async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        trace!("Sending {:?}", msg);
        let frame = CANFrame::new(msg.header, &msg.data, false, false)
            .expect("failed to convert can message to frame");
        //self.socket.write_frame(frame).await
        match self.socket.write_frame(frame).unwrap().await {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("error: {:?}", e);
                Err(std::io::Error::new(std::io::ErrorKind::Other, "no"))
            }
        }
    }

    /*async fn next(&self) -> futures::stream::Next<'_, CanMessage> {
        self.socket.next()
    }*/
    //async fn listen(&mut self, tx: tokio::sync::mpsc::UnboundedSender<CanMessage>) {
//        tokio::spawn(async move {
    //        println!("listening");
//        });
    //}
}

/*impl Stream for AsyncSocketCanNetwork {
    type Item = CanMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        println!("poll_next");
        match self.socket.read_frame() {
            Ok(frame) => {
                println!("got frame");
                let msg = CanMessage {
                    header: frame.id(),
                    data: frame.data().to_vec(),
                    bus: self.bus,
                };
                Poll::Ready(Some(msg))
            },
            _ => Poll::Pending,
        }
    }
}*/

impl Drop for AsyncSocketCanNetwork {
    fn drop(&mut self) {
        // is there anything to drop?
    }
}

impl AsyncSocketCanNetwork {
    /// id is the network interface prefix, usually 'can' or 'vcan'
    pub fn new(bus: u8, id: String) -> Self {
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
        //AsyncSocketCanNetwork { socket: Arc::new(Mutex::new(socket)), bus }
        AsyncSocketCanNetwork { socket: socket, bus }
    }
}
