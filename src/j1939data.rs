use std::collections::HashMap;

use calamine::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct J1939DARow {
    // #[serde(rename = "Revised")]
    // #[serde(rename = "PG Revised")]
    // #[serde(rename = "SP Revised")]
    // #[serde(rename = "SP to PG Map Revised")]
    #[serde(rename = "PGN")]
    pub pg: Option<u16>,

    #[serde(rename = "PG Label")]
    pub pg_label: Option<String>,
    #[serde(rename = "PG Acronym")]
    pub pg_acronym: Option<String>,

    // #[serde(rename = "PG Description")]
    // #[serde(rename = "EDP")]
    // #[serde(rename = "DP")]
    // #[serde(rename = "PF")]
    // #[serde(rename = "PS")]
    // #[serde(rename = "Multipacket")]
    #[serde(rename = "Transmission Rate")]
    pub transmission_rate: Option<String>,

    // #[serde(rename = "PG Data Length")]
    // #[serde(rename = "Default Priority")]
    // #[serde(rename = "PG Reference")]
    // #[serde(rename = "SP Position in PG")]
    #[serde(rename = "SP Start Bit")]
    pub sp_start_bit: Option<String>,

    #[serde(rename = "SPN")]
    pub spn: Option<u16>,

    #[serde(rename = "SP Label")]
    pub sp_label: Option<String>,

    #[serde(rename = "SP Description")]
    pub sp_description: Option<String>,

    // #[serde(rename = "SP Length")]
    // #[serde(rename = "Scaling")]
    // #[serde(rename = "Offset")]
    // #[serde(rename = "Data Range")]
    // #[serde(rename = "Operational Range")]
    #[serde(rename = "Unit")]
    pub unit: Option<String>,
    // #[serde(rename = "SLOT Identifier")]
    // #[serde(rename = "SLOT Name")]
    // #[serde(rename = "SP Type")]
    // #[serde(rename = "SP Reference")]
    #[serde(rename = "Scale Factor (value only)")]
    pub scale: Option<f64>,
    #[serde(rename = "Offset (value only)")]
    pub offset: Option<f64>,
    #[serde(rename = "Range Maximum (value only)")]
    pub max: Option<f64>,

    #[serde(rename = "Length Minimum (bits)")]
    pub length_min: Option<u16>,
    #[serde(rename = "Length Maximum (bits)")]
    pub length_max: Option<u16>,
    // #[serde(rename = "SP Document")]
    // #[serde(rename = "PG Document")]
    // #[serde(rename = "SP Created or Modified Date")]
    // #[serde(rename = "PG Created or Modified Date")]
    // #[serde(rename = "SP to PG Mapping Created or Modified Date")]
}
impl J1939DARow {}
pub fn load_j1939da(file: String) -> anyhow::Result<HashMap<u16, J1939DARow>> {
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
    for result in iter {
        // ignore missing spns
        if result.is_ok() {
            let data: J1939DARow = result?;
            if let Some(spn) = data.spn {
                map.insert(spn, data);
            }
        }
    }
    Ok(map)
}
