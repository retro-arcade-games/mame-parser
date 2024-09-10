use crate::{
    core::models::{
        callback_progress::{CallbackType, ProgressCallback, ProgressInfo},
        core_models::{Machine, Resource},
    },
    helpers::callback_progress_helper::get_progress_info,
};
use anyhow::Context;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;

/// Reads a resource file and processes its content to extract machine-related resources.
///
/// This function reads an XML-based resource file, processes its content, and populates a `HashMap` of `Machine` objects
/// with their associated resources. It uses an XML reader to parse the file, identifies relevant nodes,
/// and updates the machines with the extracted information.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the resource file to be read and processed.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs
///   with their associated resources.
/// - On failure: Contains an error if the file cannot be opened, read, or if there are issues processing its content.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
/// - There is an error while parsing the XML content.
///
/// # File structure
/// The `resources.dat` file format represents a structured dataset of various resources associated with arcade machines.
/// The structure is organized into `machine` elements, each representing a different resource grouping.
/// Below is the outline of the structure used for parsing this file:
///
/// - `Machine`: Represents a resource group associated with a specific machine:
///   - `name`: The unique identifier for the machine or resource group (attribute).
///     - Possible values include: `artpreview`, `bosses`, `cabinets`, `covers`, `cpanel`, `devices`,
///       `ends`, `flyers`, `gameover`, `howto`, `icons`, `logo`, `manuals`, `marquees`, `pcb`,
///       `scores`, `select`, `snap`, `titles`, `versus`, `videosnaps`, `warning`.
///
///   - `description`: A textual description of the resource group (child node).
///
///   - `roms`: A collection of `rom` elements, each representing a specific resource file associated with the machine (child nodes).
///     - Each `<rom>` element has the following attributes:
///       - `name`: The name of the resource file including the file path (e.g., `artpreview\005.png`).
///       - `size`: The size of the resource file in bytes.
///       - `crc`: The CRC32 checksum of the resource file, used for integrity verification.
///       - `sha1`: The SHA1 hash of the resource file, providing a more secure integrity check.
///
/// - `machine`: Each machine element groups together a set of related resources, identified by the `name` attribute.
/// - `description`: Provides a brief textual description of the machine or resource group.
/// - `rom`: Represents individual resource files, associated with artwork, snapshots, or other media related to the arcade machine.
///
pub fn read_resources_file(
    file_path: &str,
    progress_callback: ProgressCallback,
) -> Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>> {
    let mut machines: HashMap<String, Machine> = HashMap::new();

    let data_file_name = file_path.split('/').last().unwrap();

    // Get total elements
    progress_callback(get_progress_info(
        format!("Getting total entries for {}", data_file_name).as_str(),
    ));

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
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

    progress_callback(get_progress_info(
        format!("Reading {}", data_file_name).as_str(),
    ));

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

/// Processes an XML node to extract machine and resource information.
///
/// This function processes XML nodes from a reader, extracting relevant machine and resource information
/// and storing it in a `HashMap` of `Machine` objects. It identifies nodes related to machines and ROM resources,
/// and appropriately updates the `current_section` and `machines` with the parsed data.
///
/// # Parameters
/// - `e`: A reference to the current XML event (`BytesStart`) representing the node being processed.
/// - `reader`: A mutable reference to the `Reader` instance that reads the XML content.
/// - `current_section`: A mutable reference to an `Option<String>` representing the current section being processed.
///   It is updated with the machine or section name if a `machine` node is encountered.
/// - `machines`: A mutable reference to a `HashMap<String, Machine>` that stores the `Machine` objects
///   with their names as keys. This is updated with resources or machine information based on the XML node being processed.
///
/// # Returns
/// Returns a `Result<(), Box<dyn std::error::Error + Send + Sync>>`:
/// - On success: Returns `Ok(())` indicating that the node was processed without errors.
/// - On failure: Returns an error if there is an issue while reading or processing the XML attributes.
///
/// # Errors
/// This function will return an error if:
/// - There is a failure to decode or extract any XML attributes or content.
/// - There is a parsing issue or I/O error while processing the XML node.
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

/// Counts the total number of `<rom>` elements in an XML file content.
///
/// This function reads the content of an XML string and counts the number of `<rom>` elements
/// encountered in the file. The count represents the total number of ROM entries found within the XML structure.
///
/// # Parameters
/// - `file_content`: A `&str` representing the content of the XML file to be read and analyzed.
///
/// # Returns
/// Returns a `Result<usize, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the total number of `<rom>` elements found in the XML content.
/// - On failure: Contains an error if there is an issue reading or processing the XML content.
///
/// # Errors
/// This function will return an error if:
/// - The XML content cannot be read due to an unexpected format or malformed data.
/// - There is an I/O issue while processing the XML content.
fn count_total_elements(file_content: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
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
