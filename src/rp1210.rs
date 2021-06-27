use libloading::*;
use anyhow::*;

use super::multiqueue::*;
use super::packet::*;

struct Rp1210 {
    lib: Library,
    //send:Symbol<unsafe extern fn
}
impl Rp1210 {
    // load DLL, make connection and background thread to read all packets into queue
    pub fn loadDll(&self, id: String, connection: String, queue: MultiQueue<Packet>) -> Result<()> {
        self.lib = unsafe {
            Library::new("C:/windows/" + id + ".dll")?;
        }
            self.send = self.lib.get();
            std::thread::spawn(|| {});
        
    }
    pub fn unload() -> anyhow::Result<()> {
        self.lib.close()
    }
    pub fn send(packet: Packet) {}
}
