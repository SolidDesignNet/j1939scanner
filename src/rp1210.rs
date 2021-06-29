use anyhow::*;
use libloading::*;

use crate::j1939::packet::*;
use crate::multiqueue::*;

pub struct Rp1210 {
    lib: Library,
    bus: MultiQueue<Packet>,
}
impl Rp1210 {
    //NULN2R32
    pub fn new(id: String, the_bus: MultiQueue<Packet>) -> anyhow::Result<Rp1210> {
        Ok(Rp1210 {
            lib: unsafe { Library::new(format!("C:/windows/{}.dll", id))? },
            bus: the_bus,
        })
    }
    // load DLL, make connection and background thread to read all packets into queue
    // FIXME, return a handle to close
    pub fn run(&self, dev: u16, connection: String, queue: MultiQueue<Packet>) -> Result<()> {
        self.RP1210_ClientConnect(dev, "J1939:Baud=Auto".to_string());
        std::thread::spawn(|| {});
        Ok(())
    }
    pub fn RP1210_ClientConnect(&self, nDeviceID: u16, fpchProtocol: String) -> u16 {
        unsafe {
            let f: Symbol<unsafe extern "C" fn(u32, u16, String, u32, u32, u16) -> u16> =
                self.lib.get(b"RP1210_ClientConnect\0").unwrap();
            f(0, nDeviceID, fpchProtocol, 0, 0, 0)
        }
    }
    pub fn unload(self) -> anyhow::Result<()> {
        self.lib.close(); // FIXME
        Ok(())
    }
    pub fn send(&self, packet: &Packet) {}
}
