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
impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{:02X}{:04X}{:02X} {}",
            self.data[0],
            ((self.data[1] as u16) << 8) | (self.data[2] as u16),
            self.data[3],
            self.data
                .iter()
                .skip(8)
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
