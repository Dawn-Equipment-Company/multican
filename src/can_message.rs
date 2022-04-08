use std::fmt;

use tracing::error;

#[derive(Clone)]
pub struct CanMessage {
    /// 29 bit CAN header
    pub header: u32,
    /// Message payload
    pub data: Vec<u8>,
    /// Bus index
    pub bus: u8,
}

impl CanMessage {
    /// Gets the data length
    pub fn dlc(&self) -> usize {
        self.data.len()
    }

    /// Source address
    pub fn sa(&self) -> u8 {
        (self.header & 0x0000_00FF) as u8
    }

    /// Destination address.  Only valid if this message is destination specific
    pub fn da(&self) -> u8 {
        ((self.header & 0x0000_FF00) >> 8) as u8
    }

    /// Parameter group number
    pub fn pgn(&self) -> u16 {
        // destination specific, and out the destination address
        if ((self.header & 0x00FF_0000) >> 16) < (0xEF as u32) {
            ((self.header & 0x00FF_0000) >> 8) as u16
        } else {
            ((self.header & 0x00FF_FF00) >> 8) as u16
        }
    }

    /// Serializes a CAN message into a byte vector
    pub fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![
            ((self.header & 0xFF00_0000) >> 24) as u8,
            ((self.header & 0x00FF_0000) >> 16) as u8,
            ((self.header & 0x0000_FF00) >> 8) as u8,
            (self.header & 0x0000_00FF) as u8,
            (self.data.len() as u8),
        ];
        v.append(&mut self.data.clone());
        v
    }

    /// Deserializes a CAN message from bytes
    pub fn from_bytes(bytes: &[u8]) -> CanMessage {
        if bytes.len() < 5 {
            error!("error message too short");
        }
        let header = (u32::from(bytes[0]) << 24)
            | (u32::from(bytes[1]) << 16)
            | (u32::from(bytes[2]) << 8)
            | u32::from(bytes[3]);
        let dlc = bytes[4] as usize;
        let data = bytes[5..(dlc + 5)].to_vec();
        CanMessage {
            header,
            data,
            bus: 0,
        }
    }
}

impl fmt::Debug for CanMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let d = self.data.iter().map(|b| format!("{:X}", b)).collect::<String>().join(" ");
        write!(f, "{}:\t0x{:X}\t{:02X?}", self.bus, self.header, self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dlc_as_fn() {
        let cm = CanMessage {
            bus: 0,
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(5, cm.dlc());
    }

    #[test]
    fn verify_sa() {
        let cm = CanMessage {
            bus: 0,
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(0x55, cm.sa());
    }

    #[test]
    fn verify_da() {
        let cm = CanMessage {
            bus: 0,
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(0x44, cm.da());
    }

    #[test]
    fn verify_dest_specific_pgn() {
        let cm = CanMessage {
            bus: 0,
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(0x2300, cm.pgn());
    }

    #[test]
    fn verify_global_pgn() {
        let cm = CanMessage {
            bus: 0,
            header: 0x18EF9922,
            data: vec![1, 2, 3, 4, 5],
        };
        let pgn = cm.pgn();
        assert_eq!(0xEF99, pgn);
    }
}
