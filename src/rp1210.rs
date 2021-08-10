use anyhow::*;
use libloading::*;
use std::ffi::CString;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::*;
use std::sync::*;

use crate::j1939::packet::*;
use crate::multiqueue::*;
use libloading::os::windows::Symbol as WinSymbol;

pub const PACKET_SIZE: usize = 1600;

// TODO: break out library calls into private struct

type ClientConnectType = unsafe extern "stdcall" fn(i32, i16, *const char, i32, i32, i16) -> i16;
type SendType = unsafe extern "stdcall" fn(i16, *const u8, i16, i16, i16) -> i16;
type ReadType = unsafe extern "stdcall" fn(i16, *const u8, i16, i16) -> i16;
type CommandType = unsafe extern "stdcall" fn(u16, i16, *const u8, u16) -> i16;
type _VERSION = unsafe extern "stdcall" fn(i16, *const u8, i16, i16) -> i16;
type GetErrorType = unsafe extern "stdcall" fn(i16, *const u8) -> i16;

pub struct Rp1210 {
    lib: Library,
    bus: MultiQueue<J1939Packet>,
    running: Arc<AtomicBool>,
    id: i16,

    client_connect_fn: WinSymbol<ClientConnectType>,
    send_fn: WinSymbol<SendType>,
    read_fn: WinSymbol<ReadType>,
    send_command_fn: WinSymbol<CommandType>,
    get_error_fn: WinSymbol<GetErrorType>,
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
#[derive(Debug)]
pub struct Rp1210Dev {
    id: u32,
    name: String,
    description: String,
}
#[derive(Debug)]
pub struct Rp1210Prod {
    id: String,
    devices: Vec<Rp1210Dev>,
}

pub fn list_all_products() -> Vec<Rp1210Prod> {
    ini::ini!("c:\\Windows\\RP121032.ini")["RP1210Support"]["APIImplementations"]
        .clone()
        .unwrap()
        .split(",")
        .map(|s| {
            let id = s.to_string();
            let devices = list_devices_for_prod(&id);
            Rp1210Prod { id, devices }
        })
        .collect()
}

fn list_devices_for_prod(id: &str) -> Vec<Rp1210Dev> {
    println!("prod: {}", id);
    let ini = ini::ini!(&format!("c:\\Windows\\{}", id));
    // find device IDs for J1939
    let j1939_devices: Vec<String> = ini
        .iter()
        .filter(|(k, t)| {
            k.starts_with("ProtocolInformation") && t["ProtocolString"] == Some("J1939".to_string())
        })
        .flat_map(|(k, t)| {
            println!("    :   {}", k);
            t["Devices"]
                .clone()
                .map_or(vec![], |s| s.split(",").map(|t| t.to_string()).collect())
                .into_iter()
        })
        .collect();
    // find the specified devices
    ini.iter()
        .filter(|(k, t)| {
            k.starts_with("DeviceInformation")
                && j1939_devices.contains(&t["DeviceId"].clone().unwrap_or("X".to_string()))
        })
        .map(|(_, e)| Rp1210Dev {
            id: e["DeviceID"]
                .clone()
                .unwrap_or("0".to_string())
                .parse()
                .unwrap(),
            name: e["DeviceName"].clone().unwrap_or("Unknown".to_string()),
            description: e["DeviceDescription"]
                .clone()
                .unwrap_or("Unknown".to_string()),
        })
        .collect()
}

impl Rp1210 {
    pub fn scan() -> Vec<Rp1210Prod> {
        list_all_products()
    }
    //NULN2R32
    pub fn new(id: &str, bus: MultiQueue<J1939Packet>) -> Result<Rp1210> {
        let rp1210 = unsafe {
            let lib = Library::new(id.to_string())?;
            let client_connect: Symbol<ClientConnectType> =
                (&lib).get(b"RP1210_ClientConnect\0").unwrap();
            let send: Symbol<SendType> = (&lib).get(b"RP1210_SendMessage\0").unwrap();
            let send_command: Symbol<CommandType> = (&lib).get(b"RP1210_SendCommand\0").unwrap();
            let read: Symbol<ReadType> = (&lib).get(b"RP1210_ReadMessage\0").unwrap();
            let get_error: Symbol<GetErrorType> = (&lib).get(b"RP1210_GetErrorMsg\0").unwrap();
            Rp1210 {
                id: 0,
                running: Arc::new(AtomicBool::new(false)),
                bus: bus.clone(),
                client_connect_fn: client_connect.into_raw(),
                send_fn: send.into_raw(),
                read_fn: read.into_raw(),
                send_command_fn: send_command.into_raw(),
                get_error_fn: get_error.into_raw(),
                lib,
            }
        };
        Ok(rp1210)
    }
    // load DLL, make connection and background thread to read all packets into queue
    pub fn run(&mut self, dev: i16, connection: &str, address: u8) -> Result<i16> {
        let running = self.running.clone();
        let mut bus = self.bus.clone();
        let read = *self.read_fn;
        let rtn = self.client_connect(dev, connection, address);
        if let Ok(id) = rtn {
            std::thread::spawn(move || {
                running.store(true, Relaxed);
                let mut buf: [u8; PACKET_SIZE] = [0; PACKET_SIZE];
                while running.load(Relaxed) {
                    let p = unsafe {
                        let size = read(id, buf.as_mut_ptr(), PACKET_SIZE as i16, 1) as usize;
                        J1939Packet::new_rp1210(&buf[0..size])
                    };
                    bus.push(p);
                }
            });
        };
        rtn
    }
    pub fn stop(&self) -> Result<()> {
        self.running.store(false, Relaxed);
        Ok(())
    }
    fn send_command(&self, cmd: u16, buf: Vec<u8>) -> Result<i16> {
        self.verify_return(unsafe {
            (self.send_command_fn)(cmd, self.id, buf.as_ptr(), buf.len() as u16)
        })
    }
    fn get_error(&self, code: i16) -> Result<String> {
        let mut buf: [u8; 1024] = [0; 1024];
        unsafe {
            let size = (self.get_error_fn)(code, buf.as_mut_ptr()) as usize;
            Ok(String::from_utf8_lossy(&buf[0..size]).to_string())
        }
    }
    fn verify_return(&self, v: i16) -> Result<i16> {
        if v < 0 {
            Err(anyhow!(self.get_error(-v)?))
        } else {
            Ok(v)
        }
    }
    pub fn client_connect(
        &mut self,
        dev_id: i16,
        connection_string: &str,
        address: u8,
    ) -> Result<i16> {
        let c_to_print = CString::new(connection_string).expect("CString::new failed");
        let id = unsafe {
            (self.client_connect_fn)(0, dev_id, c_to_print.as_ptr() as *const char, 0, 0, 0)
        };
        self.id = self.verify_return(id)?;
        self.send_command(
            /*CMD_PROTECT_J1939_ADDRESS*/ 19,
            vec![
                address, 0, 0, 0xE0, 0xFF, 0, 0x81, 0, 0, /*CLAIM_BLOCK_UNTIL_DONE*/ 0,
            ],
        )?;
        self.send_command(
            /*CMD_ECHO_TRANSMITTED_MESSAGES*/ 16,
            vec![/*ECHO_ON*/ 1],
        )?;
        self.send_command(/*CMD_SET_ALL_FILTERS_STATES_TO_PASS*/ 3, vec![])?;
        Ok(id)
    }
    pub fn unload(self) -> anyhow::Result<()> {
        self.lib.close()?;
        Ok(())
    }
    pub fn send(&self, packet: &J1939Packet) -> Result<i16> {
        let buf = &packet.packet.data;
        self.verify_return(unsafe { (self.send_fn)(self.id, buf.as_ptr(), buf.len() as i16, 0, 0) })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        for p in list_all_products() {
            println!("{:?}", p);
        }
    }
}
