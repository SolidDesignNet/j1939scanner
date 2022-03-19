use anyhow::*;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::*;
use std::sync::*;

use crate::j1939::packet::*;
use crate::multiqueue::*;

pub struct Rp1210 {
    bus: MultiQueue<J1939Packet>,
    running: Arc<AtomicBool>,
}
#[allow(dead_code)]
impl Rp1210 {
    pub fn new(_id: &str, bus: MultiQueue<J1939Packet>) -> Result<Rp1210> {
        let rp1210 = Rp1210 {
            running: Arc::new(AtomicBool::new(false)),
            bus,
        };
        Ok(rp1210)
    }
    // load DLL, make connection and background thread to read all packets into queue
    pub fn run(
        &mut self,
        _dev: i16,
        _connection: &str,
        _address: u8,
    ) -> Result<Box<dyn Fn() >> {
        let running = self.running.clone();
        let mut bus = self.bus.clone();
        std::thread::spawn(move || {
            running.store(true, Relaxed);
            // example to test compile
            bus.push(J1939Packet::new(0x18DA55F9, &[0x10, 0x01]));
            todo!() // send traffic
        });
        Ok(Box::new(move || {}))
    }
    pub fn stop(&self) -> Result<()> {
        self.running.store(false, Relaxed);
        Ok(())
    }
    pub fn client_connect(
        &mut self,
        _dev_id: i16,
        _connection_string: &str,
        _address: u8,
    ) -> Result<i16> {
        todo!()
    }
    pub fn unload(self) -> anyhow::Result<()> {
        todo!()
    }
    pub fn send(&self, _packet: &J1939Packet) -> Result<i16> {
        todo!()
    }
}
