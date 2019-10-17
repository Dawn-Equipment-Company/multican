use std::fmt;

#[derive(Clone)]
pub struct CanMessage {
    pub header: u32,
    pub data: Vec<u8>,
    pub bus: u8,
}

impl CanMessage {
    pub fn dlc(&self) -> usize {
        self.data.len()
    }

    pub fn sa(&self) -> u8 {
        (self.header & 0x0000_00FF) as u8
    }

    pub fn da(&self) -> u8 {
        ((self.header & 0x0000_FF00) >> 8) as u8
    }

    pub fn pgn(&self) -> u16 {
        ((self.header & 0x00FF_FF00) >> 8) as u16
    }

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

    pub fn from_bytes(bytes: &[u8]) -> CanMessage {
        if bytes.len() < 5 {
            println!("error message too short");
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
        write!(f, "{}:\t0x{:X}\t{:?}", self.bus, self.header, self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dlc_as_fn() {
        let cm = CanMessage {
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(5, cm.dlc());
    }

    #[test]
    fn verify_sa() {
        let cm = CanMessage {
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(0x55, cm.sa());
    }

    #[test]
    fn verify_da() {
        let cm = CanMessage {
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(0x44, cm.da());
    }

    #[test]
    fn verify_pgn() {
        let cm = CanMessage {
            header: 0x18234455,
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(0x2344, cm.pgn());
    }

}
