use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::filters::{machine_names_normalization, manufacturers_normalization};
use crate::core::models::{BiosSet, DeviceRef, Disk, ExtendedData, Machine, Rom, Sample, Software};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::{self, File};
use std::io::BufReader;
use std::{collections::HashMap, error::Error};

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

    let file = File::open(file_path)?;
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
                    machine_names_normalization::normalize_name(&machine.description);
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
                    manufacturers_normalization::normalize_manufacturer(&machine.manufacturer);
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
