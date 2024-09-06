use crate::core::{
    data_cleanup::name_normalization,
    models::{
        callback_progress::{CallbackType, ProgressCallback, ProgressInfo},
        core_models::{BiosSet, DeviceRef, Disk, ExtendedData, Machine, Rom, Sample, Software},
    },
};
use anyhow::Context;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::{self, File};
use std::io::BufReader;
use std::{collections::HashMap, error::Error};

/// Reads a MAME file and processes the machine entries contained within.
///
/// This function opens and reads the specified MAME file, counting the total number of
/// machine entries, then iteratively processes each entry to construct a `HashMap` of machines.
///
/// # Parameters
/// - `file_path`: The path to the MAME file to be read.
/// - `progress_callback`: A callback function to report progress during the file processing.
///
/// # Returns
/// - `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
///   - On success: A `HashMap` where each key is a machine name and the value is the corresponding `Machine` struct.
///   - On failure: An error if the file could not be read or processed.
///
/// # Errors
/// - Returns an error if the file cannot be opened or read.
/// - Returns an error if there is an issue processing the XML content.
///
/// # File structure
/// The `mame.dat` file format represents data about arcade machines and their components.
///
/// This format is used to parse the details of various arcade machines, their associated
/// ROMs, BIOS sets, devices, software, samples, and other attributes. The following is a
/// description of the structure and elements used within this file:
///
/// # Machine
/// Represents a single arcade machine with various attributes:
/// - `name`: The unique identifier for the machine (attribute).
/// - `source_file`: Optional source file for the machine's data (attribute).
/// - `rom_of`: Indicates the ROM depends on files from another ROM to function correctly (optional, attribute).
/// - `clone_of`: Indicates the ROM is a modified version or variant of another ROM known as the parent ROM (optional, attribute).
/// - `is_bios`: Flag indicating if the machine is a BIOS (optional, attribute).
/// - `is_device`: Flag indicating if the machine is a device (optional, attribute).
/// - `runnable`: Flag indicating if the machine is runnable (optional, attribute).
/// - `is_mechanical`: Flag indicating if the machine is mechanical (optional, attribute).
/// - `sample_of`: Indicates the ROM uses specific sound samples from another ROM (optional, attribute).
/// - `description`: Textual description of the machine (optional, child node).
/// - `year`: Year of release (optional, child node).
/// - `manufacturer`: Manufacturer name (optional, child node).
///
/// # BIOS Sets
/// - `bios_sets`: List of BIOS sets related to the machine (optional, child nodes).
///   - Each `<biosset>` element includes:
///     - `name`: Name of the BIOS set (attribute).
///     - `description`: Description of the BIOS set (attribute).
///
/// # ROMs
/// - `roms`: List of ROMs associated with the machine (optional, child nodes).
///   - Each `<rom>` element includes:
///     - `name`: Name of the ROM (attribute).
///     - `size`: Size of the ROM (attribute).
///     - `merge`: Merge attribute (optional, attribute).
///     - `status`: Status attribute (optional, attribute).
///     - `crc`: CRC value (optional, attribute).
///     - `sha1`: SHA1 value (optional, attribute).
///
/// # Device References
/// - `device_refs`: List of device references related to the machine (optional, child nodes).
///   - Each `<device_ref>` element includes:
///     - `name`: Name of the device reference (attribute).
///
/// # Software List
/// - `software_list`: List of software associated with the machine (optional, child nodes).
///   - Each `<softwarelist>` element includes:
///     - `name`: Name of the software (attribute).
///
/// # Samples
/// - `samples`: List of samples associated with the machine (optional, child nodes).
///   - Each `<sample>` element includes:
///     - `name`: Name of the sample (attribute).
///
/// # Driver Status
/// - `driver_status`: Status of the machine's driver (optional, child node).
///
/// # Disks
/// - `disks`: List of disks related to the machine (optional, child nodes).
///   - Each `<disk>` element includes:
///     - `name`: Name of the disk (attribute).
///     - `sha1`: SHA1 value (optional, attribute).
///     - `merge`: Merge attribute (optional, attribute).
///     - `status`: Status attribute (optional, attribute).
///     - `region`: Region attribute (optional, attribute).
pub fn read_mame_file(
    file_path: &str,
    progress_callback: ProgressCallback,
) -> Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>> {
    let mut machines: HashMap<String, Machine> = HashMap::new();

    let data_file_name = file_path.split('/').last().unwrap();

    // Get total elements
    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Getting total entries for {}", data_file_name),
        callback_type: CallbackType::Info,
    });

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);

    // Read the file content
    let file_content = fs::read_to_string(file_path)?;

    // Count the number of machines in the file
    let total_elements = match count_total_elements(&file_content) {
        Ok(total_elements) => total_elements,
        Err(err) => {
            progress_callback(ProgressInfo {
                progress: 0,
                total: 0,
                message: format!("Couldn't get total entries for {}", data_file_name),
                callback_type: CallbackType::Error,
            });

            return Err(err.into());
        }
    };

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Reading {}", data_file_name),
        callback_type: CallbackType::Info,
    });

    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let mut buf = Vec::with_capacity(8 * 1024);

    let mut current_machine: Option<Machine> = None;

    let mut processed_count = 0;
    let batch = total_elements / 10;

    loop {
        match xml_reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                process_node(e, &mut xml_reader, &mut current_machine)?;
            }
            Ok(Event::Empty(ref e)) => {
                process_node(e, &mut xml_reader, &mut current_machine)?;
            }
            Ok(Event::End(ref e)) => match e.name() {
                b"machine" => {
                    if let Some(machine) = current_machine.take() {
                        machines
                            .entry(machine.name.clone())
                            .or_insert_with(|| machine);
                    }

                    // Increase processed count
                    processed_count += 1;
                    // Progress callback
                    if processed_count % batch == 0 {
                        progress_callback(ProgressInfo {
                            progress: processed_count as u64,
                            total: total_elements as u64,
                            message: String::from(""),
                            callback_type: CallbackType::Progress,
                        });
                    }
                }
                _ => (),
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => (),
        }
        buf.clear();
    }

    progress_callback(ProgressInfo {
        progress: processed_count as u64,
        total: total_elements as u64,
        message: format!("{} loaded successfully", data_file_name),
        callback_type: CallbackType::Finish,
    });

    Ok(machines)
}

/// Processes an XML node and updates the current machine with the parsed data.
///
/// This function handles different types of XML elements relevant to the structure of
/// the machine data. It initializes new machines, reads their attributes, and adds
/// various components (such as ROMs, BIOS sets, and software lists) to the machine structure.
///
/// # Parameters
/// - `e`: A reference to the `BytesStart` event representing the start of an XML element.
/// - `reader`: A mutable reference to the `Reader` used to read the XML data.
/// - `current_machine`: A mutable reference to an `Option<Machine>`, which will be updated
///   with the parsed machine data.
///
/// # Returns
/// Returns a `Result<(), Box<dyn Error + Send + Sync>>`:
/// - On success: Indicates the node was processed without errors.
/// - On failure: Contains an error if there were issues reading the XML or updating the machine data.
fn process_node(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<BufReader<File>>,
    current_machine: &mut Option<Machine>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match e.name() {
        b"machine" => {
            let mut machine = Machine {
                name: String::new(),
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
                bios_sets: vec![],
                roms: vec![],
                device_refs: vec![],
                software_list: vec![],
                samples: vec![],
                driver_status: None,
                languages: vec![],
                players: None,
                series: None,
                category: None,
                subcategory: None,
                is_mature: None,
                history_sections: vec![],
                disks: vec![],
                extended_data: None,
                resources: vec![],
            };
            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => machine.name = attr.unescape_and_decode_value(reader)?,
                    b"sourcefile" => {
                        machine.source_file = Some(attr.unescape_and_decode_value(reader)?)
                    }
                    b"romof" => machine.rom_of = Some(attr.unescape_and_decode_value(reader)?),
                    b"cloneof" => machine.clone_of = Some(attr.unescape_and_decode_value(reader)?),
                    b"isbios" => {
                        machine.is_bios = Some(attr.unescape_and_decode_value(reader)? == "yes")
                    }
                    b"isdevice" => {
                        machine.is_device = Some(attr.unescape_and_decode_value(reader)? == "yes")
                    }
                    b"runnable" => {
                        machine.runnable = Some(attr.unescape_and_decode_value(reader)? == "yes")
                    }
                    b"ismechanical" => {
                        machine.is_mechanical =
                            Some(attr.unescape_and_decode_value(reader)? == "yes")
                    }
                    b"sampleof" => {
                        machine.sample_of = Some(attr.unescape_and_decode_value(reader)?)
                    }
                    _ => {}
                }
            }
            // Set is_parent flag in Extended Data
            if machine.extended_data.is_none() {
                machine.extended_data = Some(ExtendedData::default());
            }
            machine.extended_data.as_mut().unwrap().is_parent = Some(true);
            if machine.clone_of.is_some() || machine.rom_of.is_some() {
                machine.extended_data.as_mut().unwrap().is_parent = Some(false);
            }

            *current_machine = Some(machine);
        }
        b"description" => {
            if let Some(ref mut machine) = current_machine {
                machine.description = Some(reader.read_text(b"description", &mut Vec::new())?);
                // Set normalized name in Extended Data
                let refactored_name =
                    name_normalization::normalize_machine_name(&machine.description);
                machine.extended_data.as_mut().unwrap().name = Some(refactored_name.clone());
            }
        }
        b"year" => {
            if let Some(ref mut machine) = current_machine {
                machine.year = Some(reader.read_text(b"year", &mut Vec::new())?);
                // If year contains ? or is empty then set year in Extended Data as Unknown
                if machine.year.as_ref().unwrap().contains('?')
                    || machine.year.as_ref().unwrap().is_empty()
                {
                    machine.extended_data.as_mut().unwrap().year = Some("Unknown".to_string());
                } else {
                    machine.extended_data.as_mut().unwrap().year = machine.year.clone();
                }
            }
        }
        b"manufacturer" => {
            if let Some(ref mut machine) = current_machine {
                machine.manufacturer = Some(reader.read_text(b"manufacturer", &mut Vec::new())?);
                // Set normalized manufacturer in Extended Data
                let normalized_manufacturer =
                    name_normalization::normalize_manufacturer_name(&machine.manufacturer);
                machine.extended_data.as_mut().unwrap().manufacturer =
                    Some(normalized_manufacturer.clone());
            }
        }
        b"biosset" => {
            let mut bios_set = BiosSet {
                name: String::new(),
                description: String::new(),
            };

            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => bios_set.name = attr.unescape_and_decode_value(reader)?,
                    b"description" => {
                        bios_set.description = attr.unescape_and_decode_value(reader)?
                    }
                    _ => {}
                }
            }
            if let Some(ref mut machine) = current_machine {
                machine.bios_sets.push(bios_set);
            }
        }
        b"rom" => {
            let mut rom = Rom {
                name: String::new(),
                merge: None,
                size: 0,
                crc: None,
                sha1: None,
                status: None,
            };
            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => rom.name = attr.unescape_and_decode_value(reader)?,
                    b"merge" => rom.merge = Some(attr.unescape_and_decode_value(reader)?),
                    b"size" => {
                        rom.size = attr.unescape_and_decode_value(reader)?.parse().unwrap_or(0)
                    }
                    b"crc" => rom.crc = Some(attr.unescape_and_decode_value(reader)?),
                    b"sha1" => rom.sha1 = Some(attr.unescape_and_decode_value(reader)?),
                    b"status" => rom.status = Some(attr.unescape_and_decode_value(reader)?),
                    _ => {}
                }
            }
            if let Some(ref mut machine) = current_machine {
                machine.roms.push(rom);
            }
        }
        b"device_ref" => {
            let mut device_ref = DeviceRef {
                name: String::new(),
            };

            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => device_ref.name = attr.unescape_and_decode_value(reader)?,
                    _ => {}
                }
            }
            if let Some(ref mut machine) = current_machine {
                machine.device_refs.push(device_ref);
            }
        }
        b"softwarelist" => {
            let mut software = Software {
                name: String::new(),
            };

            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => software.name = attr.unescape_and_decode_value(reader)?,
                    _ => {}
                }
            }
            if let Some(ref mut machine) = current_machine {
                machine.software_list.push(software);
            }
        }
        b"sample" => {
            let mut sample = Sample {
                name: String::new(),
            };

            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => sample.name = attr.unescape_and_decode_value(reader)?,
                    _ => {}
                }
            }
            if let Some(ref mut machine) = current_machine {
                machine.samples.push(sample);
            }
        }
        b"disk" => {
            let mut disk = Disk {
                name: String::new(),
                sha1: None,
                merge: None,
                status: None,
                region: None,
            };
            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => disk.name = attr.unescape_and_decode_value(reader)?,
                    b"sha1" => disk.sha1 = Some(attr.unescape_and_decode_value(reader)?),
                    b"merge" => disk.merge = Some(attr.unescape_and_decode_value(reader)?),
                    b"status" => disk.status = Some(attr.unescape_and_decode_value(reader)?),
                    b"region" => disk.region = Some(attr.unescape_and_decode_value(reader)?),
                    _ => {}
                }
            }
            if let Some(ref mut machine) = current_machine {
                machine.disks.push(disk);
            }
        }
        b"driver" => {
            let mut driver_status = String::new();
            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"status" => driver_status = attr.unescape_and_decode_value(reader)?,
                    _ => {}
                }
            }
            if let Some(ref mut machine) = current_machine {
                machine.driver_status = Some(driver_status);
            }
        }
        _ => (),
    }

    Ok(())
}

/// Counts the total number of `<machine>` elements in the provided XML content.
///
/// This function parses the given XML content line by line and counts how many `<machine>` elements
/// are present. The count is used to determine the total number of machines represented in the XML.
///
/// # Parameters
/// - `file_content`: A `&str` containing the entire content of the XML file as a string.
///
/// # Returns
/// Returns a `Result<usize, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the total number of `<machine>` elements found in the XML content.
/// - On failure: Contains an error if there are issues while reading or parsing the XML content.
///
/// # Errors
/// This function will return an error if:
/// - There are I/O errors or issues while reading and parsing the XML content.
fn count_total_elements(file_content: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let mut reader = Reader::from_str(file_content);
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(8 * 1024);
    let mut count = 0;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"machine" => {
                count += 1;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                // Return the error instead of printing it
                return Err(Box::new(e));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(count)
}
