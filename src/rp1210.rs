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
type GET_ERROR = unsafe extern "stdcall" fn(i16, *const u8) -> i16;

pub struct Rp1210 {
    lib: Library,
    bus: MultiQueue<Packet>,
    running: Arc<AtomicBool>,
    id: i16,

    clientConnect: WinSymbol<CC>,
    send: WinSymbol<SEND>,
    read: WinSymbol<READ>,
    sendCommand: WinSymbol<COMMAND>,
    get_error: WinSymbol<GET_ERROR>,
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
    pub fn new(id: &str, the_bus: MultiQueue<Packet>) -> Result<Rp1210> {
        let dll = format!("{}", id);
        let rp1210 = unsafe {
            let lib = Library::new(dll)?;
            let cc: Symbol<CC> = (&lib).get(b"RP1210_ClientConnect\0").unwrap();
            let send: Symbol<SEND> = (&lib).get(b"RP1210_SendMessage\0").unwrap();
            let sendCommand: Symbol<COMMAND> = (&lib).get(b"RP1210_SendCommand\0").unwrap();
            let read: Symbol<READ> = (&lib).get(b"RP1210_ReadMessage\0").unwrap();
            let get_error: Symbol<GET_ERROR> = (&lib).get(b"RP1210_GetErrorMsg\0").unwrap();
            Rp1210 {
                id: 0,
                running: Arc::new(AtomicBool::new(false)),
                bus: the_bus,
                clientConnect: cc.into_raw(),
                send: send.into_raw(),
                read: read.into_raw(),
                sendCommand: sendCommand.into_raw(),
                get_error: get_error.into_raw(),
                lib,
            }
        };
        Ok(rp1210)
    }
    // load DLL, make connection and background thread to read all packets into queue
    // FIXME, return a handle to close
    pub fn run(&mut self, dev: i16, connection: &str, address: u8) -> Result<()> {
        let running = self.running.clone();
        let mut bus = self.bus.clone();
        let read = *self.read;
        let id = self.RP1210_ClientConnect(dev, connection, address)?;
        std::thread::spawn(move || {
            running.store(true, Relaxed);
            let buf = &mut Vec::with_capacity(PACKET_SIZE as usize);
            while running.load(Relaxed) {
                let p = {
                    unsafe {
                        let size = read(id, buf.as_mut_ptr(), PACKET_SIZE, 1);
                        buf.set_len(size as usize);
                        Packet::new(buf)
                    }
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
    fn sendCommand(&self, cmd: u16, buf: Vec<u8>) -> Result<i16> {
        unsafe {
            self.verify_return((self.sendCommand)(
                cmd,
                self.id,
                buf.as_ptr(),
                buf.len() as u16,
            ))
        }
    }
    fn get_error(&self, code: i16) -> Result<String> {
        let mut buf = Vec::with_capacity(1024);
        unsafe {
            let size = (self.get_error)(code, buf.as_ptr());
            buf.set_len(size as usize);
            Ok(String::from_utf8_lossy(&buf[..]).to_string())
        }
    }
    fn verify_return(&self, v: i16) -> Result<i16> {
        if v < 0 {
            let msg = self.get_error(-v)?;
            Err(anyhow!(msg))
        } else {
            Ok(v)
        }
    }
    pub fn RP1210_ClientConnect(
        &mut self,
        nDeviceID: i16,
        fpchProtocol: &str,
        address: u8,
    ) -> Result<i16> {
        let c_to_print = CString::new(fpchProtocol).expect("CString::new failed");
        let id = unsafe { (self.clientConnect)(0, 1, c_to_print.as_ptr() as *const char, 0, 0, 0) };
        self.id = self.verify_return(id)?;
        self.sendCommand(
            /*CMD_PROTECT_J1939_ADDRESS*/ 19,
            vec![
                address, 0, 0, 0xE0, 0xFF, 0, 0x81, 0, 0, /*CLAIM_BLOCK_UNTIL_DONE*/ 0,
            ],
        )?;
        self.sendCommand(
            /*CMD_ECHO_TRANSMITTED_MESSAGES*/ 16,
            vec![/*ECHO_ON*/ 1],
        )?;
        self.sendCommand(/*CMD_SET_ALL_FILTERS_STATES_TO_PASS*/ 3, vec![])?;
        Ok(id)
    }
    pub fn unload(self) -> anyhow::Result<()> {
        self.lib.close()?;
        Ok(())
    }
    pub fn send(&self, packet: &Packet) -> Result<()> {
        todo!()
    }
}
