extern crate gio;
extern crate gtk;

use std::collections::HashMap;
use std::io;
use std::process;

use calamine::{open_workbook, Error, RangeDeserializer, Reader, Xlsx};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct J1939DARow {
    // #[serde(rename = "Revised")]
    // #[serde(rename = "PG Revised")]
    // #[serde(rename = "SP Revised")]
    // #[serde(rename = "SP to PG Map Revised")]
    #[serde(rename = "PGN")]
    pub pg: u16,

    #[serde(rename = "PG Label")]
    pub pg_label: String,
    #[serde(rename = "PG Acronym")]
    pub pg_acronym: String,

    // #[serde(rename = "PG Description")]
    // #[serde(rename = "EDP")]
    // #[serde(rename = "DP")]
    // #[serde(rename = "PF")]
    // #[serde(rename = "PS")]
    // #[serde(rename = "Multipacket")]
    #[serde(rename = "Transmission Rate")]
    pub transmission_rate: String,

    // #[serde(rename = "PG Data Length")]
    // #[serde(rename = "Default Priority")]
    // #[serde(rename = "PG Reference")]
    // #[serde(rename = "SP Position in PG")]
    #[serde(rename = "SP Start Bit")]
    pub sp_start_bit: String,

    #[serde(rename = "SPN")]
    pub spn: u16,

    #[serde(rename = "SP Label")]
    pub sp_label: String,

    #[serde(rename = "SP Description")]
    pub sp_description: String,

    // #[serde(rename = "SP Length")]
    // #[serde(rename = "Scaling")]
    // #[serde(rename = "Offset")]
    // #[serde(rename = "Data Range")]
    // #[serde(rename = "Operational Range")]
    #[serde(rename = "Unit")]
    pub unit: String,
    // #[serde(rename = "SLOT Identifier")]
    // #[serde(rename = "SLOT Name")]
    // #[serde(rename = "SP Type")]
    // #[serde(rename = "SP Reference")]
    #[serde(rename = "Scale Factor (value only)")]
    pub scale: f64,
    #[serde(rename = "Offset (value only)")]
    pub offset: f64,
    #[serde(rename = "Range Maximum (value only)")]
    pub max: f64,

    #[serde(rename = "Length Minimum (bits)")]
    pub length_min: u16,
    #[serde(rename = "Length Maximum (bits)")]
    pub length_max: u16,
    // #[serde(rename = "SP Document")]
    // #[serde(rename = "PG Document")]
    // #[serde(rename = "SP Created or Modified Date")]
    // #[serde(rename = "PG Created or Modified Date")]
    // #[serde(rename = "SP to PG Mapping Created or Modified Date")]
}
impl J1939DARow {
    // pub fn pgn_label(&self) -> Option<String> {
    //     self.pgn.map(|p| format!("{} ({:X})", p, p))
    // }
    // pub fn spn_label(&self) -> Option<String> {
    //     self.spn.map(|p| format!("{} ({:X})", p, p))
    // }
}
pub fn load_j1939da(file: String) {
    let mut excel: Xlsx<_> = open_workbook(file)?;

    let range = excel
        .worksheet_range("SPs & PGs")
        .ok_or(Error::Msg("Cannot find 'SPs & PGs'"))??;
    let mut iter = RangeDeserializerBuilder::new().from_range(&r)?;
    if let Some(result) = iter.next() {
        let row: J1939DARow = result?;
        println!("data:{:?}", data);
    }
    //        let mut iter = r.deserialize();
    // for row in iter {
    //     let data: J1939DARow = row?;
    //     println!("data:{:?}", data);
    // }
}
