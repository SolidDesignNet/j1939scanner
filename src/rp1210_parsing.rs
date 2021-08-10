use anyhow::*;

#[derive(Debug)]
pub struct Rp1210Dev {
    pub id: u32,
    pub name: String,
    pub description: String,
}
#[derive(Debug)]
pub struct Rp1210Prod {
    pub id: String,
    pub devices: Vec<Rp1210Dev>,
}

pub fn list_all_products() -> Result<Vec<Rp1210Prod>> {
    Ok(ini::Ini::load_from_file("c:\\Windows\\RP121032.ini")?
        .get_from(Some("RP1210Support"), "APIImplementations")
        .unwrap_or("")
        .split(",")
        .map(|s| Rp1210Prod {
            id: s.to_string(),
            devices: list_devices_for_prod(s).unwrap(),
        })
        .collect())
}

fn list_devices_for_prod(id: &str) -> Result<Vec<Rp1210Dev>> {
    let ini = ini::Ini::load_from_file(&format!("c:\\Windows\\{}.ini", id))?;

    // find device IDs for J1939
    let j1939_devices: Vec<&str> = ini
        .iter()
        // find J1939 protocol description
        .filter(|(section, properties)| {
            section.unwrap_or("").starts_with("ProtocolInformation")
                && properties.get("ProtocolString") == Some("J1939")
        })
        // which device ids support J1939?
        .flat_map(|(_, properties)| {
            properties
                .get("Devices")
                .map_or(vec![], |s| s.split(",").collect())
        })
        .collect();

    // find the specified devices
    Ok(ini
        .iter()
        .filter(|(section, properties)| {
            section.unwrap().starts_with("DeviceInformation")
                && j1939_devices.contains(&properties.get("DeviceID").unwrap_or("X"))
        })
        .map(|(_, properties)| Rp1210Dev {
            id: properties.get("DeviceId").unwrap_or("0").parse().unwrap(),
            name: properties
                .get("DeviceName")
                .unwrap_or("Unknown")
                .to_string(),
            description: properties
                .get("DeviceDescription")
                .unwrap_or("Unknown")
                .to_string(),
        })
        .collect())
}
