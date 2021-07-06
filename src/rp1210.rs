use anyhow::*;
use libloading::*;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::*;
use std::sync::*;

use crate::j1939::packet::*;
use crate::multiqueue::*;

pub struct Rp1210 {
    lib: Library,
    bus: MultiQueue<Packet>,
    running: Arc<AtomicBool>,
}
impl Rp1210 {
    //NULN2R32
    pub fn new(id: &str, the_bus: MultiQueue<Packet>) -> anyhow::Result<Rp1210> {
        Ok(Rp1210 {
            running: Arc::new(AtomicBool::new(false)),
            lib: unsafe { Library::new(format!("C:/windows/{}.dll", id))? },
            bus: the_bus,
        })
    }
    // load DLL, make connection and background thread to read all packets into queue
    // FIXME, return a handle to close
    pub fn run(&self, dev: u16, connection: &str) -> Result<()> {
        self.RP1210_ClientConnect(dev, connection);
        let running = self.running.clone();
        let mut bus = self.bus.clone();
        std::thread::spawn(move || {
            running.store(true, Relaxed);
            while running.load(Relaxed) {
                let p = Packet::new();
                bus.push(p);
            }
        });
        Ok(())
    }
    pub fn stop(&self) {
        self.running.store(false, Relaxed)
    }
    pub fn RP1210_ClientConnect(&self, nDeviceID: u16, fpchProtocol: &str) -> u16 {
        unsafe {
            let f: Symbol<unsafe extern "C" fn(u32, u16, &str, u32, u32, u16) -> u16> =
                self.lib.get(b"RP1210_ClientConnect\0").unwrap();
            f(0, nDeviceID, fpchProtocol, 0, 0, 0)
        }
    }
    pub fn unload(self) -> anyhow::Result<()> {
        self.lib.close()?;
        Ok(())
    }
    pub fn send(&self, packet: &Packet) {
        todo!()
    }
}
