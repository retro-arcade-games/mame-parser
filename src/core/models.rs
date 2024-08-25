use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    pub name: String,
    pub source_file: Option<String>,
    pub rom_of: Option<String>,
    pub clone_of: Option<String>,
    pub is_bios: Option<bool>,
    pub is_device: Option<bool>,
    pub runnable: Option<bool>,
    pub is_mechanical: Option<bool>,
    pub sample_of: Option<String>,
    pub description: Option<String>,
    pub year: Option<String>,
    pub manufacturer: Option<String>,
    pub bios_sets: Vec<BiosSet>,
    pub roms: Vec<Rom>,
    pub device_refs: Vec<DeviceRef>,
    pub software_list: Vec<Software>,
    pub samples: Vec<Sample>,
    pub driver_status: Option<String>,
    pub languages: Vec<String>,
    pub players: Option<String>,
    pub series: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub is_mature: Option<bool>,
    pub history_sections: Vec<HistorySection>,
    pub disks: Vec<Disk>,
    pub extended_data: Option<ExtendedData>,
    pub resources: Vec<Resource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiosSet {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rom {
    pub name: String,
    pub size: u64,
    pub merge: Option<String>,
    pub status: Option<String>,
    pub crc: Option<String>,
    pub sha1: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRef {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Software {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sample {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disk {
    pub name: String,
    pub sha1: Option<String>,
    pub merge: Option<String>,
    pub status: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistorySection {
    pub name: String,
    pub text: String,
    pub order: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExtendedData {
    // Normalization of the description field
    pub name: Option<String>,
    // Normalization of the manufacturer field
    pub manufacturer: Option<String>,
    // Normalization of the players field
    pub players: Option<String>,
    // Indicates if the machine is a parent
    pub is_parent: Option<bool>,
    // Normalization of the year field
    pub year: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub type_: String,
    pub name: String,
    pub size: u64,
    pub crc: String,
    pub sha1: String,
}
