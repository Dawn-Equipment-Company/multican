
use crate::{CanMessage, CanNetwork};
use socketcan::{CANFrame, CANSocket};
use std::time;

/// SocketCAN adapter for mulitcan
pub struct SocketCanNetwork {
    socket: CANSocket,
    pub bus: u8,
}

impl CanNetwork for SocketCanNetwork {
    fn send(&self, msg: CanMessage) {
        trace!("Sending {:?}", msg);
        let frame = CANFrame::new(msg.header, &msg.data, false, false)
            .expect("failed to convert can message to frame");
        self.socket
            .write_frame(&frame)
            .expect("Failed to write can message");
    }

    fn recv(&self) -> Option<CanMessage> {
        match self.socket.read_frame() {
            Ok(frame) => {
                let msg = CanMessage {
                    header: frame.id(),
                    data: frame.data().to_vec(),
                    bus: self.bus,
                };
                Some(msg)
            }
            _ => None,
        }
    }
}

impl Drop for SocketCanNetwork {
    fn drop(&mut self) {
        println!("TODO: Closing socketcan connection");
    }
}

impl SocketCanNetwork {
    pub fn new(bus: u8) -> Self {
        // remember, the owa4x starts at can1 instead of can0
        let bus = bus + 1;
        debug!("Initializing bus #{}", bus);
        let bus_id = format!("can{}", bus);
        debug!("Opening bus number {} - id: {}", bus, bus_id);
        let socket = CANSocket::open(&bus_id).expect("Failed to open bus");
        socket
            .set_read_timeout(time::Duration::from_millis(100))
            .expect("Failed to set socketcan read timeout");
        socket
            .set_nonblocking(true)
            .expect("Failed to set socketcan socket to nonblocking");

        SocketCanNetwork { socket, bus }
    }
}
