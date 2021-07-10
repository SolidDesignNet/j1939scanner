use std::fmt::*;

#[derive(Debug)]
pub struct Packet {
    pub data: Vec<u8>,
}
impl Clone for Packet {
    fn clone(&self) -> Self {
        Packet {
            data: self.data.clone(),
        }
    }
}

pub struct J1939Packet {
    packet: Packet,
}
impl Clone for J1939Packet {
    fn clone(&self) -> Self {
        J1939Packet {
            packet: self.packet.clone(),
        }
    }
}
impl J1939Packet {
    pub fn new(data: &[u8]) -> J1939Packet {
        J1939Packet {
            packet: Packet::new(data),
        }
    }
    pub fn length(&self) -> usize {
        self.packet.data.len() - 11
    }
}
impl Display for J1939Packet {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{} {} [{}]{}",
            self.time(),
            self.header(),
            self.length(),
            {
                let mut s = String::new();
                for byte in self.data() {
                    write!(&mut s, " {:02X}", byte).expect("Unable to write");
                }
                s
            }
        )
    }
}
impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            self.data
                .iter()
                .fold(String::new(), |a, &n| a + &n.to_string() + ", ")
        )
    }
}
impl Packet {
    pub fn new(data: &[u8]) -> Packet {
        Packet {
            data: data.to_vec(),
        }
    }
}
impl J1939Packet {
    fn time(&self) -> u64 {
        let timestamp = (0xFF000000 & (self.packet.data[0] as u64) << 24)
            | (0xFF0000 & (self.packet.data[1] as u64) << 16)
            | (0xFF00 & (self.packet.data[2] as u64) << 8)
            | (0xFF & (self.packet.data[3] as u64));
        //timestamp *= self.timestampWeight;
        timestamp
    }
    fn echo(&self) -> bool {
        self.packet.data[4] != 0
    }
    fn source(&self) -> u8 {
        self.packet.data[9]
    }
    fn pgn(&self) -> u32 {
        let mut pgn = ((self.packet.data[7] as u32 & 0xFF) << 16)
            | ((self.packet.data[6] as u32 & 0xFF) << 8)
            | (self.packet.data[5] as u32 & 0xFF);
        if pgn < 0xF000 {
            let destination = self.packet.data[10] as u32;
            pgn |= destination;
        }
        pgn
    }
    fn priority(&self) -> u8 {
        self.packet.data[8] & 0x07
    }
    fn header(&self) -> String {
        format!(
            "{:06X}{:02X}",
            ((self.priority() as u32) << 18) | self.pgn(),
            self.source()
        )
    }
    fn data(&self) -> Vec<u8> {
        self.packet.data.clone().into_iter().skip(11).collect()
    }
}
