use std::collections::HashMap;

use anyhow::Result;
use calamine::*;
use serde::Deserialize;

pub mod packet;

fn bool_from_string<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "Yes" => Ok(Some(true)),
        "No" => Ok(Some(false)),
        _other => Ok(None),
    }
}

#[derive(Debug, Deserialize)]
pub struct J1939DARow {
    // #[serde(alias = "Revised")]
    // #[serde(alias = "PG Revised")]
    // #[serde(alias = "SP Revised")]
    // #[serde(alias = "SP to PG Map Revised")]
    #[serde(alias = "PGN")]
    pub pg: Option<u32>,

    #[serde(alias = "PG Label")]
    pub pg_label: Option<String>,
    #[serde(alias = "PG Acronym")]
    pub pg_acronym: Option<String>,

    #[serde(alias = "PG Description")]
    pub pg_description: Option<String>,

    #[serde(alias = "EDP")]
    pub edp: Option<String>,
    #[serde(alias = "DP")]
    pub dp: Option<String>,
    #[serde(alias = "PF")]
    pub pf: Option<String>,
    #[serde(alias = "PS")]
    pub ps: Option<String>,

    #[serde(alias = "Multipacket", deserialize_with = "bool_from_string")]
    pub multipacket: Option<bool>,

    #[serde(alias = "Transmission Rate")]
    pub transmission_rate: Option<String>,

    // #[serde(alias = "PG Data Length")]
    // #[serde(alias = "Default Priority")]
    // #[serde(alias = "PG Reference")]
    // #[serde(alias = "SP Position in PG")]
    #[serde(alias = "SP Start Bit")]
    pub sp_start_bit: Option<String>,

    #[serde(alias = "SPN")]
    pub spn: Option<u32>,

    #[serde(alias = "SP Label")]
    pub sp_label: Option<String>,

    #[serde(alias = "SP Description")]
    pub sp_description: Option<String>,

    // #[serde(alias = "SP Length")]
    // #[serde(alias = "Scaling")]
    // #[serde(alias = "Offset")]
    // #[serde(alias = "Data Range")]
    // #[serde(alias = "Operational Range")]
    #[serde(alias = "Unit")]
    pub unit: Option<String>,
    // #[serde(alias = "SLOT Identifier")]
    // #[serde(alias = "SLOT Name")]
    // #[serde(alias = "SP Type")]
    // #[serde(alias = "SP Reference")]
    #[serde(alias = "Scale Factor\r\n(value only)")]
    pub scale: Option<f64>,
    #[serde(alias = "Offset\r\n(value only)")]
    pub offset: Option<f64>,
    #[serde(alias = "Range Maximum\r\n(value only)")]
    pub max: Option<f64>,

    #[serde(alias = "Length Minimum\r\n(bits)")]
    pub length_min: Option<u16>,
    #[serde(alias = "Length Maximum\r\n(bits)")]
    pub length_max: Option<u16>,
    // #[serde(alias = "SP Document")]
    // #[serde(alias = "PG Document")]
    // #[serde(alias = "SP Created or Modified Date")]
    // #[serde(alias = "PG Created or Modified Date")]
    // #[serde(alias = "SP to PG Mapping Created or Modified Date")]
}
impl J1939DARow {}
pub fn load_j1939da(file: &str) -> Result<HashMap<u32, J1939DARow>> {
    let start = std::time::Instant::now();
    let mut excel: Xlsx<_> = open_workbook(file)?;
    let range = excel
        .worksheet_range("SPs & PGs")
        .ok_or(Error::Msg("Cannot find 'SPs & PGs'"))??;

    // skip the first 3 rows
    let subrange = range.range((3, 0), range.end().unwrap());
    let iter = RangeDeserializerBuilder::new()
        .has_headers(true)
        .from_range(&subrange)?;
    let mut map = HashMap::new();
    let mut errors = 0;
    let mut count = 0;
    for result in iter {
        count += 1;
        // ignore missing spns
        if result.is_ok() {
            let data: J1939DARow = result?;
            if let Some(spn) = data.spn {
                map.insert(spn, data);
            }
        } else {
            errors += 1;
            //println!("FAIL {}: {:?}", errors, result);
        }
    }
    println!(
        "J1939DA parsing {} ms {} errors of {} total rows.",
        start.elapsed().as_millis(),
        errors,
        count
    );
    Ok(map)
}
