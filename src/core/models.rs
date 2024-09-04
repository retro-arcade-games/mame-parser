use serde::{Deserialize, Serialize};

/// MAME machine, including all relevant metadata and resources.
///
/// The `Machine` struct stores detailed information about a specific MAME machine,
/// including its configuration, associated ROMs, BIOS sets, devices, and other related metadata.
/// This structure is used in parsing, processing, and exporting MAME-related data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    /// The name of the machine.
    pub name: String,
    /// The source file associated with the machine (optional).
    pub source_file: Option<String>,
    /// Specifies the ROM that this machine is a variant of (optional).
    pub rom_of: Option<String>,
    /// Specifies the parent machine if this is a clone (optional).
    pub clone_of: Option<String>,
    /// Indicates if the machine is a BIOS set (optional).
    pub is_bios: Option<bool>,
    /// Indicates if the machine is a device (optional).
    pub is_device: Option<bool>,
    /// Indicates if the machine is runnable (optional).
    pub runnable: Option<bool>,
    /// Indicates if the machine is mechanical (optional).
    pub is_mechanical: Option<bool>,
    /// Specifies the sample set associated with the machine (optional).
    pub sample_of: Option<String>,
    /// A description of the machine (optional).
    pub description: Option<String>,
    /// The year the machine was released (optional).
    pub year: Option<String>,
    /// The manufacturer of the machine (optional).
    pub manufacturer: Option<String>,
    /// A list of BIOS sets associated with the machine.
    pub bios_sets: Vec<BiosSet>,
    /// A list of ROMs required by the machine.
    pub roms: Vec<Rom>,
    /// A list of device references associated with the machine.
    pub device_refs: Vec<DeviceRef>,
    /// A list of software lists associated with the machine.
    pub software_list: Vec<Software>,
    /// A list of samples used by the machine.
    pub samples: Vec<Sample>,
    /// The driver status of the machine (optional).
    pub driver_status: Option<String>,
    /// A list of supported languages for the machine.
    pub languages: Vec<String>,
    /// Indicates the number of players supported (optional).
    pub players: Option<String>,
    /// The series to which the machine belongs (optional).
    pub series: Option<String>,
    /// The category of the machine (optional).
    pub category: Option<String>,
    /// The subcategory of the machine (optional).
    pub subcategory: Option<String>,
    /// Indicates if the machine contains mature content (optional).
    pub is_mature: Option<bool>,
    /// A list of history sections associated with the machine.
    pub history_sections: Vec<HistorySection>,
    /// A list of disk data associated with the machine.
    pub disks: Vec<Disk>,
    /// Additional normalized data not present in the original MAME data (optional).
    pub extended_data: Option<ExtendedData>,
    /// A list of external resources, such as images and videos, associated with the machine.
    pub resources: Vec<Resource>,
}

impl Machine {
    /// Creates a new `Machine` instance with the specified name.
    pub fn new(name: String) -> Self {
        Machine {
            name,
            source_file: None,
            rom_of: None,
            clone_of: None,
            is_bios: None,
            is_device: None,
            runnable: None,
            is_mechanical: None,
            sample_of: None,
            description: None,
            year: None,
            manufacturer: None,
            bios_sets: Vec::new(),
            roms: Vec::new(),
            device_refs: Vec::new(),
            software_list: Vec::new(),
            samples: Vec::new(),
            driver_status: None,
            languages: Vec::new(),
            players: None,
            series: None,
            category: None,
            subcategory: None,
            is_mature: None,
            history_sections: Vec::new(),
            disks: Vec::new(),
            extended_data: Some(Default::default()),
            resources: Vec::new(),
        }
    }
    /// Combines the metadata of this machine with another machine.
    pub fn combine(&mut self, other: &Machine) {
        if self.source_file.is_none() {
            self.source_file = other.source_file.clone();
        }
        if self.rom_of.is_none() {
            self.rom_of = other.rom_of.clone();
        }
        if self.clone_of.is_none() {
            self.clone_of = other.clone_of.clone();
        }
        if self.is_bios.is_none() {
            self.is_bios = other.is_bios;
        }
        if self.is_device.is_none() {
            self.is_device = other.is_device;
        }
        if self.runnable.is_none() {
            self.runnable = other.runnable;
        }
        if self.is_mechanical.is_none() {
            self.is_mechanical = other.is_mechanical;
        }
        if self.sample_of.is_none() {
            self.sample_of = other.sample_of.clone();
        }
        if self.description.is_none() {
            self.description = other.description.clone();
        }
        if self.year.is_none() {
            self.year = other.year.clone();
        }
        if self.manufacturer.is_none() {
            self.manufacturer = other.manufacturer.clone();
        }
        if self.driver_status.is_none() {
            self.driver_status = other.driver_status.clone();
        }
        if self.players.is_none() {
            self.players = other.players.clone();
        }
        if self.series.is_none() {
            self.series = other.series.clone();
        }
        if self.category.is_none() {
            self.category = other.category.clone();
        }
        if self.subcategory.is_none() {
            self.subcategory = other.subcategory.clone();
        }
        if self.is_mature.is_none() {
            self.is_mature = other.is_mature;
        }

        self.bios_sets.extend(other.bios_sets.clone());
        self.roms.extend(other.roms.clone());
        self.device_refs.extend(other.device_refs.clone());
        self.software_list.extend(other.software_list.clone());
        self.samples.extend(other.samples.clone());
        self.languages.extend(other.languages.clone());
        self.history_sections.extend(other.history_sections.clone());
        self.disks.extend(other.disks.clone());
        self.resources.extend(other.resources.clone());

        match (&mut self.extended_data, &other.extended_data) {
            (Some(self_data), Some(other_data)) => {
                self_data.combine(other_data);
            }
            (None, Some(other_data)) => {
                self.extended_data = Some(other_data.clone());
            }
            _ => {}
        }
    }
}

/// BIOS set associated with a MAME machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiosSet {
    /// The name of the BIOS set.
    pub name: String,
    /// A description of the BIOS set.
    pub description: String,
}

/// ROM file associated with a MAME machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rom {
    /// The name of the ROM file.
    pub name: String,
    /// The size of the ROM file in bytes.
    pub size: u64,
    /// Indicates if the ROM is merged with another ROM (optional).
    pub merge: Option<String>,
    /// The status of the ROM (optional).
    pub status: Option<String>,
    /// The CRC32 hash of the ROM file (optional).
    pub crc: Option<String>,
    /// The SHA-1 hash of the ROM file (optional).
    pub sha1: Option<String>,
}

/// Device reference associated with a MAME machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRef {
    /// The name of the device.
    pub name: String,
}

/// Software list associated with a MAME machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    /// The name of the software.
    pub name: String,
}

/// Sample file associated with a MAME machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    /// The name of the sample file.
    pub name: String,
}

/// Disk data associated with a MAME machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    /// The name of the disk.
    pub name: String,
    /// The SHA-1 hash of the disk file (optional).
    pub sha1: Option<String>,
    /// Indicates if the disk is merged with another disk (optional).
    pub merge: Option<String>,
    /// The status of the disk (optional).
    pub status: Option<String>,
    /// The region associated with the disk (optional).
    pub region: Option<String>,
}

/// Historical section or trivia associated with a MAME machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySection {
    /// The name of the history section.
    pub name: String,
    /// The text content of the history section.
    pub text: String,
    /// The order in which this section should appear.
    pub order: usize,
}

/// Represents additional normalized data for a MAME machine.
///
/// This structure is used to store normalized or additional data that is not present
/// in the original MAME files but is useful for further processing or display.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExtendedData {
    /// Normalized name of the machine (optional).
    pub name: Option<String>,
    /// Normalized manufacturer of the machine (optional).
    pub manufacturer: Option<String>,
    /// Normalized number of players (optional).
    pub players: Option<String>,
    /// Indicates if the machine is a parent (optional).
    pub is_parent: Option<bool>,
    /// Normalized release year (optional).
    pub year: Option<String>,
}

impl ExtendedData {
    /// Combines the metadata of this extended data with another extended data.
    pub fn combine(&mut self, other: &ExtendedData) {
        if self.name.is_none() {
            self.name = other.name.clone();
        }
        if self.manufacturer.is_none() {
            self.manufacturer = other.manufacturer.clone();
        }
        if self.players.is_none() {
            self.players = other.players.clone();
        }
        if self.is_parent.is_none() {
            self.is_parent = other.is_parent;
        }
        if self.year.is_none() {
            self.year = other.year.clone();
        }
    }
}

/// External resource associated with a MAME machine, such as images or videos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// The type of the resource (e.g., "image", "video").
    pub type_: String,
    /// The name of the resource.
    pub name: String,
    /// The size of the resource in bytes.
    pub size: u64,
    /// The CRC32 hash of the resource.
    pub crc: String,
    /// The SHA-1 hash of the resource.
    pub sha1: String,
}
