use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::models::Machine;
use crate::core::models::Resource;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;

pub fn read_resources_file(
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

    let mut current_section: Option<String> = None;

    let mut processed_count = 0;
    let batch = total_elements / 10;

    loop {
        match xml_reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                process_node(e, &mut xml_reader, &mut current_section, &mut machines)?;
            }
            Ok(Event::Empty(ref e)) => {
                process_node(e, &mut xml_reader, &mut current_section, &mut machines)?;
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
    current_section: &mut Option<String>,
    machines: &mut HashMap<String, Machine>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match e.name() {
        b"machine" => {
            let mut section_name: Option<String> = None;
            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => section_name = Some(attr.unescape_and_decode_value(reader)?),
                    _ => {}
                }
            }

            *current_section = section_name;
        }
        b"rom" => {
            let mut resource = Resource {
                type_: String::new(),
                name: String::new(),
                size: 0,
                crc: String::new(),
                sha1: String::new(),
            };
            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => resource.name = attr.unescape_and_decode_value(reader)?,
                    b"size" => {
                        resource.size = attr.unescape_and_decode_value(reader)?.parse().unwrap_or(0)
                    }
                    b"crc" => resource.crc = attr.unescape_and_decode_value(reader)?,
                    b"sha1" => resource.sha1 = attr.unescape_and_decode_value(reader)?,
                    _ => {}
                }
            }
            // Get the machine name based on the rom name
            let splitted = resource.name.split("\\").collect::<Vec<&str>>();

            if splitted.len() < 2 {
                return Ok(());
            }

            let resource_type = splitted[0].to_string();
            let machine_name = splitted[1].split(".").next().unwrap_or_default();

            // If exists section name then add the information to the machine if the machine exists
            if let Some(section_name) = current_section {
                // Check if the resource type is the same as the section name
                // Avoid adding non arcade resources to the machines
                if *section_name == resource_type {
                    // Get or insert machine
                    let machine = machines
                        .entry(machine_name.to_owned())
                        .or_insert_with(|| Machine::new(machine_name.to_owned()));
                    // Add the resource to the machine
                    resource.type_ = section_name.clone();
                    machine.resources.push(resource);
                }
            }
        }
        _ => (),
    }

    Ok(())
}

pub fn count_total_elements(file_content: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let mut reader = Reader::from_str(file_content);
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(8 * 1024);
    let mut count = 0;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(ref e)) if e.name() == b"rom" => {
                count += 1;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(Box::new(e));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(count)
}
