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
                tokio::select! {
                    Some(Send(m)) = send_rx.recv() => {
                        inner.send(m).await.unwrap()
                    }
                    Some(Next(tx)) = next_rx.recv() => {
                        if let Some(m) = inner.next().await {
                            tx.send(m).unwrap();
                        }
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
        let res = self
            .send_tx
            .send(Send(msg))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e));

        res
    }

    async fn next(&self) -> Option<CanMessage> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

        self.next_tx
            .send(Next(reply_tx))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
            .unwrap();

        if let Ok(m) = reply_rx.await {
            Some(m)
        } else {
            None
        }
    }
}

/// SocketCAN adapter for mulitcan
struct AsyncSocketCanNetworkImpl {
    socket_tx: SplitSink<CANSocket, CANFrame>,
    socket_rx: SplitStream<CANSocket>,
    pub bus: u8,
}

impl AsyncSocketCanNetworkImpl {
    async fn send(&mut self, msg: CanMessage) -> Result<(), std::io::Error> {
        trace!("Sending {:?}", msg);
        let frame = CANFrame::new(msg.header, &msg.data, false, false)
            .expect("failed to convert can message to frame");
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
