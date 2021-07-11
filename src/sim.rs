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
impl Rp1210 {
    //NULN2R32
    pub fn new(id: &str, the_bus: MultiQueue<J1939Packet>) -> Result<Rp1210> {
        let rp1210 = Rp1210 {
            running: Arc::new(AtomicBool::new(false)),
            bus: the_bus,
        };
        Ok(rp1210)
    }
    // load DLL, make connection and background thread to read all packets into queue
    pub fn run(&mut self, dev: i16, connection: &str, address: u8) -> Result<i16> {
        let running = self.running.clone();
        let mut bus = self.bus.clone();
        std::thread::spawn(move || {
            running.store(true, Relaxed);
            todo!() // send traffic
        });
        Ok(0)
    }
    pub fn stop(&self) -> Result<()> {
        self.running.store(false, Relaxed);
        Ok(())
    }
    pub fn client_connect(
        &mut self,
        dev_id: i16,
        connection_string: &str,
        address: u8,
    ) -> Result<i16> {
        todo!()
    }
    pub fn unload(self) -> anyhow::Result<()> {
        todo!()
    }
    pub fn send(&self, packet: &Packet) -> Result<i16> {
        todo!()
    }
}
