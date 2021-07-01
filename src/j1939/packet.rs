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
}
