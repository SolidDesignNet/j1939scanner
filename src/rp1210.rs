use anyhow::*;
use libloading::*;
use std::ffi::CString;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::*;
use std::sync::*;

use crate::j1939::packet::*;
use crate::multiqueue::*;
use libloading::os::windows::Symbol as WinSymbol;

pub const PACKET_SIZE: i16 = 1600;

type CC = unsafe extern "stdcall" fn(i32, i16, *const char, i32, i32, i16) -> i16;
type SEND = unsafe extern "stdcall" fn(i16, *const u8, i16, i16, i16) -> i16;
type READ = unsafe extern "stdcall" fn(i16, *const u8, i16, i16) -> i16;
type COMMAND = unsafe extern "stdcall" fn(u16, i16, *const u8, u16) -> i16;
type VERSION = unsafe extern "stdcall" fn(i16, *const u8, i16, i16) -> i16;

pub struct Rp1210 {
    lib: Library,
    bus: MultiQueue<Packet>,
    running: Arc<AtomicBool>,
    id: i16,

    clientConnect: WinSymbol<CC>,
    send: WinSymbol<SEND>,
    read: WinSymbol<READ>,
    sendCommand: WinSymbol<COMMAND>,
    /*
      short RP1210_GetErrorMsg(short errCode, byte[] fpchMessage);
      short RP1210_ReadMessage(short nClientID, byte[] fpchAPIMessage, short nBufferSize, short nBlockOnSend);
      short RP1210_SendCommand(short nCommandNumber, short nClientID, byte[] fpchClientCommand, short nMessageSize);
      short RP1210_SendMessage(short nClientID,
                             byte[] fpchClientMessage,
                             short nMessageSize,
                             short nNotifyStatusOnTx,
                             short nBlockOnSend);

    */
}
impl Rp1210 {
    //NULN2R32
    pub fn new(id: &str, the_bus: MultiQueue<Packet>) -> anyhow::Result<Rp1210> {
        let dll = format!("{}", id);
        println!("Loading: {}", dll);
        let lib = unsafe { Library::new(dll)? };
        let rp1210 = unsafe {
            let cc: Symbol<CC> = (&lib).get(b"RP1210_ClientConnect\0").unwrap();
            let send: Symbol<SEND> = (&lib).get(b"RP1210_SendMessage\0").unwrap();
            let sendCommand: Symbol<COMMAND> = (&lib).get(b"RP1210_SendCommand\0").unwrap();
            let read: Symbol<READ> = (&lib).get(b"RP1210_ReadMessage\0").unwrap();
            Rp1210 {
                id: 0,
                running: Arc::new(AtomicBool::new(false)),
                bus: the_bus,
                clientConnect: cc.into_raw(),
                send: send.into_raw(),
                read: read.into_raw(),
                sendCommand: sendCommand.into_raw(),
                lib,
            }
        };
        println!("all found");
        Ok(rp1210)
    }
    // load DLL, make connection and background thread to read all packets into queue
    // FIXME, return a handle to close
    pub fn run(&mut self, dev: i16, connection: &str, address: u8) -> Result<()> {
        println!("About to connect {} {}", dev, connection);
        let id = self.RP1210_ClientConnect(dev, connection, address);
        println!("Client Connect: {}", id);
        let running = self.running.clone();
        let mut bus = self.bus.clone();
        let read = *self.read;
        std::thread::spawn(move || {
            println!("running: {}", id);
            running.store(true, Relaxed);
            let buf = &mut Vec::with_capacity(PACKET_SIZE as usize);
            while running.load(Relaxed) {
                println!("read: {}", id);
                let p = unsafe {
                    let size = read(id, buf.as_mut_ptr(), PACKET_SIZE, 1);
                    println!("size {}", size);
                    buf.set_len(size as usize);
                    Packet::new(buf)
                };
                println!("p: {:?}", p);
                bus.push(p);
            }
        });
        Ok(())
    }
    pub fn stop(&self) {
        self.running.store(false, Relaxed)
    }
    fn sendCommand(&self, cmd: u16, buf: Vec<u8>) -> i16 {
        unsafe { (self.sendCommand)(cmd, self.id, buf.as_ptr(), buf.len() as u16) }
    }
    pub fn RP1210_ClientConnect(&mut self, nDeviceID: i16, fpchProtocol: &str, address: u8) -> i16 {
        let c_to_print = CString::new(fpchProtocol).expect("CString::new failed");
        let id = unsafe {
            //let i = (*self.clientConnect)(0, nDeviceID, j1939, 0, 0, 0);
            // let cc: Symbol<unsafe extern "stdcall" fn(i32, i16, *const char, i32, i32, i16) -> i16> =
            //     self.lib.get(b"RP1210_ClientConnect\0").unwrap();
            (self.clientConnect)(0, 1, c_to_print.as_ptr() as *const char, 0, 0, 0)
        };
        self.id = id;
        println!("cc {}", id);
        println!(
            "prot {}",
            self.sendCommand(
                /*CMD_PROTECT_J1939_ADDRESS*/ 19,
                vec![address, 0, 0, 0xE0, 0xFF, 0, 0x81, 0, 0, /*CLAIM_BLOCK_UNTIL_DONE*/ 0,],
            )
        );
        println!(
            "echo {}",
            self.sendCommand(
                /*CMD_ECHO_TRANSMITTED_MESSAGES*/ 16,
                vec![/*ECHO_ON*/ 1],
            )
        );
        println!(
            "all {}",
            self.sendCommand(/*CMD_SET_ALL_FILTERS_STATES_TO_PASS*/ 3, vec![])
        );
        id
    }
    pub fn unload(self) -> anyhow::Result<()> {
        self.lib.close()?;
        Ok(())
    }
    pub fn send(&self, packet: &Packet) {
        todo!()
    }
}
