use crate::{AsyncCanNetwork, CanMessage};
use async_trait::async_trait;
use futures::{stream::SplitSink, stream::SplitStream, SinkExt, StreamExt};
use socketcan::CANFrame;
use tokio::sync::mpsc::Sender;
use tokio_socketcan::CANSocket;
use tracing::{debug, trace};

#[derive(Debug)]
struct Send(CanMessage);

#[derive(Debug)]
struct Next(tokio::sync::oneshot::Sender<CanMessage>);

#[derive(Clone, Debug)]
pub struct AsyncSocketCanNetwork {
    send_tx: Sender<Send>,
    next_tx: Sender<Next>,
}

impl AsyncSocketCanNetwork {
    pub fn new(bus: u8, id: &'static str) -> Self {
        let (send_tx, mut send_rx) = tokio::sync::mpsc::channel(10);
        let (next_tx, mut next_rx) = tokio::sync::mpsc::channel(10);

        debug!("Initializing bus #{}", bus);
        let bus_id = format!("{}{}", id, bus);
        debug!("Opening bus number {} - id: {}", bus, bus_id);
        let socket = CANSocket::open(&bus_id).expect("Failed to open bus");

        let (socket_tx, socket_rx) = socket.split();

        let mut inner = AsyncSocketCanNetworkImpl {
            socket_tx,
            socket_rx,
            bus,
        };

        tokio::spawn(async move {
            loop {
                println!("loop start");

                tokio::select! {
                    Some(Send(m)) = send_rx.recv() => {
                        println!("SEND COMMAND received");
                        inner.send(m).await.unwrap()
                    }
                    Some(Next(tx)) = next_rx.recv() => {
                        println!("next replied");
                        if let Some(m) = inner.next().await {
                            println!("next inner replied");
                            tx.send(m).unwrap();
                        } else  {
                            println!("next got NONe");
                        }
                    }
                    else => {
                        println!("else");
                    }
                }
            }
        });

        Self { send_tx, next_tx }
    }
}

#[async_trait]
impl AsyncCanNetwork for AsyncSocketCanNetwork {
    async fn send(&self, msg: CanMessage) -> Result<(), std::io::Error> {
        // self.inner.lock().await.send(msg).await

        // println!("sendingd ldkfjlsdjf {:?}", msg);

        let res = self
            .send_tx
            // .send(Command::Send(msg))
            .send(Send(msg))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e));

        println!("manaaged to send to inner");

        res
    }

    async fn next(&self) -> Option<CanMessage> {
        // println!("next called");
        // let r = self.inner.lock().await.next().await;
        // println!("next released");
        // r

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

        self.next_tx
            // .send(Command::Next(reply_tx))
            .send(Next(reply_tx))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
            .unwrap();

        if let Ok(m) = reply_rx.await {
            println!("some");
            Some(m)
        } else {
            println!("NOne");
            None
        }
    }
}

/// SocketCAN adapter for mulitcan
struct AsyncSocketCanNetworkImpl {
    // pub socket: CANSocket,
    socket_tx: SplitSink<CANSocket, CANFrame>,
    socket_rx: SplitStream<CANSocket>,
    pub bus: u8,
}

impl AsyncSocketCanNetworkImpl {
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        trace!("Sending {:?}", msg);
        let frame = CANFrame::new(msg.header, &msg.data, false, false)
            .expect("failed to convert can message to frame");
        // self.socket_tx.write_frame(frame).unwrap();
        self.socket_tx.send(frame).await.unwrap();
        Ok(())
    }

    async fn next(&mut self) -> Option<CanMessage> {
        if let Some(Ok(frame)) = self.socket_rx.next().await {
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

// impl Stream for AsyncSocketCanNetworkImpl {
//     type Item = CanMessage;
//
//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
//         println!("poll_next");
//         match self.socket.poll_next_unpin(cx) {
//             Poll::Ready(Some(Ok(frame))) => {
//                 println!("got frame");
//                 Poll::Ready(Some(CanMessage {
//                     header: frame.id(),
//                     data: frame.data().to_vec(),
//                     bus: self.bus,
//                 }))
//             }
//             Poll::Ready(Some(Err(_e))) => Poll::Ready(None),
//             _ => Poll::Pending,
//         }
//     }
// }

// impl AsyncSocketCanNetworkImpl {
//     /// id is the network interface prefix, usually 'can' or 'vcan'
//     pub fn new(bus: u8, id: &'static str) -> Self {
//         debug!("Initializing bus #{}", bus);
//         let bus_id = format!("{}{}", id, bus);
//         debug!("Opening bus number {} - id: {}", bus, bus_id);
//         let socket = CANSocket::open(&bus_id).expect("Failed to open bus");
//         /*socket
//             .set_read_timeout(time::Duration::from_millis(100))
//             .expect("Failed to set socketcan read timeout");
//         socket
//             .set_nonblocking(true)
//             .expect("Failed to set socketcan socket to nonblocking");*/
//         debug!("opened {}", bus_id);
//         //let s2 = CANSocket::open("can2").expect("failed to open can2");
//         //let mut x = socket.merge(s2);
//         //socket.split();
//         AsyncSocketCanNetworkImpl { socket, bus }
//     }
// }
