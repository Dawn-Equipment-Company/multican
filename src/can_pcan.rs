use can::{CanMessage, CanNetwork};
use pcan_basic_sys as pcan;

pub struct PcanNetwork {}

impl CanNetwork for PcanNetwork {
    fn send(&mut self, msg: CanMessage) {
        trace!("Sending {:?}", msg);
        let mut d: [u8; 8] = Default::default();
        d.copy_from_slice(&msg.data[0..8]);
        let mut pcan_msg = pcan::TPCANMsg {
            ID: msg.header,
            MSGTYPE: pcan::PCAN_MESSAGE_EXTENDED as u8,
            LEN: msg.data.len() as u8,
            DATA: d,
        };
        unsafe {
            let r = pcan::CAN_Write(pcan::PCAN_USBBUS1 as u16, &mut pcan_msg);
            if r != pcan::PCAN_ERROR_OK {
                error!("result: {}", r);
            }
        }
    }

    fn recv(&mut self) -> Option<CanMessage> {
        let d: [u8; 8] = Default::default();
        let mut rx: pcan::TPCANMsg = pcan::TPCANMsg {
            ID: 0,
            MSGTYPE: 0,
            LEN: 0,
            DATA: d,
        };
        let mut ts: pcan::TPCANTimestamp = pcan::TPCANTimestamp {
            millis: 0,
            millis_overflow: 0,
            micros: 0,
        };
        let mut result: Option<CanMessage> = None;
        unsafe {
            let r = pcan::CAN_Read(pcan::PCAN_USBBUS1 as u16, &mut rx, &mut ts);
            if r == pcan::PCAN_ERROR_OK {
                result = Some(CanMessage {
                    header: rx.ID,
                    bus: 0,
                    data: rx.DATA.to_vec(),
                });
            }
        }
        result
    }
}

impl PcanNetwork {
    fn connect_bus(baud: u16) -> Self {
        debug!("Initializing pcan at {} baud", baud);
        unsafe {
            let r = pcan::CAN_Initialize(pcan::PCAN_USBBUS1 as u16, baud as u16, 0, 0, 0);
            if r != pcan::PCAN_ERROR_OK {
                panic!("Error initializing pcan: {}", r);
            } else {
                debug!("Successfully initialized PCAN");
            }
        }
        PcanNetwork {}
    }

    pub fn new() -> Self {
        connect_bus(pcan::PCAN_BAUD_500K)
    }

    pub fn new_250() -> Self {
        connect_bus(pcan::PCAN_BAUD_250K)
    }
}

impl Drop for PcanNetwork {
    fn drop(&mut self) {
        debug!("Dropping PCAN network");
        unsafe {
            let r = pcan::CAN_Uninitialize(pcan::PCAN_USBBUS1 as u16);
        }
    }
}
