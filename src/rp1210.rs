use anyhow::*;
use libloading::*;

use crate::j1939::packet::*;
use crate::multiqueue::*;

pub struct Rp1210<'a> {
    lib: Library,
    bus: &'a mut MultiQueue<Packet>,
    running: bool,
}
impl<'a> Rp1210<'a> {
    //NULN2R32
    pub fn new(id: String, the_bus: &'a mut MultiQueue<Packet>) -> anyhow::Result<Rp1210> {
        Ok(Rp1210 {
            running: false,
            lib: unsafe { Library::new(format!("C:/windows/{}.dll", id))? },
            bus: the_bus,
        })
    }
    // load DLL, make connection and background thread to read all packets into queue
    // FIXME, return a handle to close
    pub fn run(&'static mut self, dev: u16, connection: String) -> Result<()> {
        self.RP1210_ClientConnect(dev, connection);
        std::thread::spawn(move || {
            self.running = true;
            while self.running {
                let p = Packet::new();
                self.bus.push(p);
            }
        });
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
    pub fn send(&self, packet: &Packet) {
        todo!()
    }
}
