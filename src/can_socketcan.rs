use crate::{CanMessage, CanNetwork};
use socketcan::{CANFrame, CANSocket};
use std::time;
use std::thread;
use std::sync::{Arc, Mutex};

/// SocketCAN adapter for mulitcan
pub struct SocketCanNetwork {
    socket: Arc<Mutex<CANSocket>>,
    pub bus: u8,
    rx_queue: Arc<Mutex<Vec<CanMessage>>>,
}

impl CanNetwork for SocketCanNetwork {
    fn send(&self, msg: CanMessage) {
        trace!("Sending {:?}", msg);
        let frame = CANFrame::new(msg.header, &msg.data, false, false)
            .expect("failed to convert can message to frame");
        self.socket
            .lock().unwrap()
            .write_frame(&frame)
            .expect("Failed to write can message");
    }

    fn recv(&self) -> Vec<CanMessage> {
        self.rx_queue.lock().unwrap().drain(..).collect()
    }
}

impl Drop for SocketCanNetwork {
    fn drop(&mut self) {
        // is there anything to drop?
    }
}

impl SocketCanNetwork {
    /// id is the network interface prefix, usually 'can' or 'vcan'
    pub fn new(bus: u8, id: &'static str) -> Self {
        debug!("Initializing bus #{}", bus);
        let bus_id = format!("{}{}", id, bus);
        debug!("Opening bus number {} - id: {}", bus, bus_id);
        let socket = CANSocket::open(&bus_id).expect("Failed to open bus");
        socket
            .set_read_timeout(time::Duration::from_millis(100))
            .expect("Failed to set socketcan read timeout");
        socket
            .set_nonblocking(false)
            .expect("Failed to set socketcan socket to blocking");
        socket
            .set_read_timeout(std::time::Duration::from_millis(10))
            .expect("Failed to set read timeout");
        let socket = Arc::new(Mutex::new(socket));
        let rx_socket = socket.clone();

        let rx_queue = Arc::new(Mutex::new(Vec::new()));
        let queue = rx_queue.clone();

        let _handle = thread::spawn(move || {
            debug!("Starting rx thread for {}", bus_id);
            loop {
                let s = rx_socket.lock().unwrap();
                match s.read_frame() {
                    Ok(frame) => {
                        let msg = CanMessage {
                            header: frame.id(),
                            data: frame.data().to_vec(),
                            bus: bus,
                        };
                        trace!("RX: {:?}", msg);
                        queue.lock().unwrap().push(msg);
                    },
                    _ => {
                        // read_frame will return WouldBlock if there's no data available.  that
                        // shuts down our thread if we .unwrap() it, so just do nothing and wait
                        // for the next one
                    },
                }
            }
        });

        SocketCanNetwork { socket, bus, rx_queue }
    }
}
