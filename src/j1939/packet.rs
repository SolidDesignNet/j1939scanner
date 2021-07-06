#[derive(Debug, Copy, Clone)]
pub struct Packet {
    head: [u8; 8],
    data: [u8; 8],
}
impl Packet {
    pub fn new() -> Packet {
        Packet {
            head: [1, 2, 3, 4, 5, 6, 7, 8],
            data: [1, 2, 3, 4, 5, 6, 7, 8],
        }
    }
    pub fn format(&self) -> String {
        format!(
            "{:02X}{:04X}{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            self.head[0],
            ((self.head[1] as u16) << 8) | (self.head[2] as u16),
            self.head[3],
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
        )
    }
}
