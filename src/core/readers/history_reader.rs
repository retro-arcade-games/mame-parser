use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::models::{HistorySection, Machine};
use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;

/// Reads and processes a history XML file to extract machine data and history sections.
///
/// This function reads a specified history XML file line by line using an XML parser,
/// extracts relevant machine information such as system names and history sections,
/// and populates a `HashMap` where the keys are machine names and the values are their
/// corresponding `Machine` structs. Progress updates are provided through a callback function.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the XML file to be read and processed.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs
///   with their associated history sections.
/// - On failure: Contains an error if the file cannot be opened, read, or if there are issues processing its content.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
/// - The XML content is malformed or cannot be parsed correctly.
///
/// # File structure
/// The XML file follows this general structure:
///
/// `<entry>`
/// - Represents an individual entry in the XML file containing details about a game.
///
/// `<systems>`
/// - Contains a list of systems that run the game.
/// - Each system is represented by a `<system>` element with the `name` attribute.
/// ```xml
/// <system name="system_name" />
/// <!-- ... other systems ... -->
/// ```
///
/// `<software>`
/// - Contains information about software related to the game.
/// - Each software item is represented by an `<item>` element with `list` and `name` attributes.
/// ```xml
/// <item list="list_name" name="software_name" />
/// ```
///
/// `<text>`
/// - Contains various sections of text about the game. The possible sections are:
///   - **DESCRIPTION**: Provides a general description of the game.
///   - **TECHNICAL**: Details technical aspects or specifications of the game.
///   - **TRIVIA**: Contains trivia or interesting facts about the game.
///   - **UPDATES**: Lists updates or changes made to the game.
///   - **SCORING**: Details on scoring or how the game is scored.
///   - **TIPS AND TRICKS**: Offers tips and tricks for playing the game.
///   - **SERIES**: Information about the game series or franchise.
///   - **STAFF**: Lists the staff or developers involved with the game.
///   - **PORTS**: Details on different ports or versions of the game.
///   - **CONTRIBUTE**: Information on how to contribute or support the game.
///
/// `</entry>`

pub fn read_history_file(
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
    let file_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file content: {}", file_path))?;

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

    let mut current_entry: Option<HistoryEntry> = None;

    let mut processed_count = 0;
    let batch = total_elements / 10;

    loop {
        match xml_reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if let Some(new_entry) = process_node(e, &mut xml_reader)? {
                    if let Some(ref mut entry) = current_entry {
                        entry.names.extend(new_entry.names);
                        entry.sections.extend(new_entry.sections);
                    } else {
                        current_entry = Some(new_entry);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                if let Some(new_entry) = process_node(e, &mut xml_reader)? {
                    if let Some(ref mut entry) = current_entry {
                        entry.names.extend(new_entry.names);
                        entry.sections.extend(new_entry.sections);
                    } else {
                        current_entry = Some(new_entry);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name() {
                b"entry" => {
                    if let Some(entry) = current_entry.take() {
                        for name in entry.names {
                            // Get or insert machine
                            let machine = machines
                                .entry(name.clone())
                                .or_insert_with(|| Machine::new(name));
                            // Add the history to the machine
                            machine.history_sections = entry.sections.clone();
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
                        // Reset current entry
                        current_entry = None;
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

/// Processes an XML node and returns an optional `HistoryEntry` based on its content.
///
/// This function processes a single XML node (`BytesStart`) and extracts relevant data,
/// such as system names and text sections, to populate a `HistoryEntry` struct.
/// Depending on the type of node (`entry`, `system`, or `text`), the function will initialize
/// or update a `HistoryEntry` object with the corresponding information.
///
/// # Parameters
/// - `e`: A reference to a `BytesStart` event representing the current XML node being processed.
/// - `reader`: A mutable reference to an XML `Reader` that reads from a buffered file input.
///
/// # Returns
/// Returns a `Result<Option<HistoryEntry>, Box<dyn std::error::Error + Send + Sync>>`:
/// - `Ok(Some(HistoryEntry))`: If a relevant entry node is processed successfully.
/// - `Ok(None)`: If the node is not relevant for creating or updating a `HistoryEntry`.
/// - `Err`: If an error occurs while reading or parsing the XML content.
///
/// # Errors
/// This function can return an error if:
/// - An attribute of a node cannot be decoded correctly.
/// - Reading the text content of a `text` node fails.
fn process_node(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<BufReader<File>>,
) -> Result<Option<HistoryEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let mut current_entry: Option<HistoryEntry> = None;

    match e.name() {
        b"entry" => {
            current_entry = Some(HistoryEntry::new());
        }
        b"system" => {
            let mut system_name = String::new();
            let attrs = e.attributes().map(|a| a.unwrap());
            for attr in attrs {
                match attr.key {
                    b"name" => system_name = attr.unescape_and_decode_value(reader)?,
                    _ => {}
                }
            }
            current_entry = Some(HistoryEntry::new());
            if let Some(ref mut entry) = current_entry {
                entry.names.push(system_name.clone());
            }
        }
        b"text" => {
            let text = reader.read_text(b"text", &mut Vec::new())?;
            let sections = parse_text(&text);
            current_entry = Some(HistoryEntry::new());
            if let Some(ref mut entry) = current_entry {
                entry.sections = sections;
            }
        }
        _ => (),
    }

    Ok(current_entry)
}

/// Parses a given text into a list of `HistorySection` structures based on predefined section headers.
///
/// This function reads the provided text line by line and identifies predefined section headers to
/// split the text into multiple sections. Each section is represented by a `HistorySection` struct,
/// which includes the section name, content, and order. The function trims whitespace and processes
/// each section's content until the next header is found.
///
/// # Parameters
/// - `text`: A `&str` representing the full text to be parsed into different sections.
///
/// # Returns
/// Returns a `Vec<HistorySection>` containing all parsed sections from the input text:
/// - Each `HistorySection` contains the section name, its corresponding text, and its order in the document.
/// - If the input text does not contain any known section headers, the function will treat all the text
///   as part of a default "description" section.
///
/// # Section Headers
/// The function recognizes the following section headers:
/// - "- DESCRIPTION -"
/// - "- TECHNICAL -"
/// - "- TRIVIA -"
/// - "- UPDATES -"
/// - "- SCORING -"
/// - "- TIPS AND TRICKS -"
/// - "- SERIES -"
/// - "- STAFF -"
/// - "- PORTS -"
/// - "- CONTRIBUTE -"
///
/// # Errors
/// The function does not return an error but may produce an empty vector if the input text is empty or does not match any recognized sections.
///
fn parse_text(text: &str) -> Vec<HistorySection> {
    let mut current_section_name = String::new();
    let mut sections = Vec::new();
    let document_sections = [
        "- DESCRIPTION -",
        "- TECHNICAL -",
        "- TRIVIA -",
        "- UPDATES -",
        "- SCORING -",
        "- TIPS AND TRICKS -",
        "- SERIES -",
        "- STAFF -",
        "- PORTS -",
        "- CONTRIBUTE -",
    ];

    let mut current_section_text = String::new();
    let mut order = 1;

    for line in text.lines() {
        if document_sections.contains(&line) {
            if !current_section_text.is_empty() {
                if current_section_name == "" {
                    current_section_name = "description".to_string();
                }
                sections.push(HistorySection {
                    name: current_section_name.clone(),
                    text: current_section_text.trim().to_string(),
                    order,
                });
                current_section_text.clear();
            }

            current_section_name = line.to_string().replace('-', "").trim().to_lowercase();
            order = get_section_order(line);
        } else {
            current_section_text.push_str(&(line.to_string() + "\n"));
        }
    }

    if !current_section_text.is_empty() {
        sections.push(HistorySection {
            name: current_section_name.clone(),
            text: current_section_text.trim().to_string(),
            order,
        });
    }

    sections
}

/// Determines the order of a given section in a predefined list of sections.
///
/// This function takes a section name as input and returns an order number (starting from 1)
/// that represents its position in a predefined sequence of sections. If the section name does
/// not match any of the predefined sections, the function returns `0`.
///
/// # Parameters
/// - `section`: A `&str` representing the name of the section whose order is to be determined.
///
/// # Returns
/// Returns a `usize` representing the order of the section:
/// - If the section matches one of the predefined names, the function returns its corresponding order number.
/// - If the section does not match any of the predefined names, the function returns `0`.
///
fn get_section_order(section: &str) -> usize {
    match section {
        "- DESCRIPTION -" => 1,
        "- TECHNICAL -" => 2,
        "- TRIVIA -" => 3,
        "- UPDATES -" => 4,
        "- SCORING -" => 5,
        "- TIPS AND TRICKS -" => 6,
        "- SERIES -" => 7,
        "- STAFF -" => 8,
        "- PORTS -" => 9,
        "- CONTRIBUTE -" => 10,
        _ => 0,
    }
}

/// Counts the total number of elements in a string based on the presence of specific XML tags (`<entry>`).
///
/// This function reads the content of a string representing an XML document line by line
/// and counts the number of `<entry>` tags found. The count represents the total number of
/// elements or entries in the XML content.
///
/// # Parameters
/// - `file_content`: A `&str` representing the XML content to be read and analyzed.
///
/// # Returns
/// Returns a `Result<usize, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the total number of `<entry>` tags found, representing the total entries in the XML content.
/// - On failure: Contains an error if the XML content cannot be read or parsed due to format or encoding issues.
///
/// # Errors
/// This function will return an error if:
/// - There are issues reading or parsing the XML content due to invalid format or encoding.
/// - The content is not a valid XML structure or contains unexpected characters.
///
fn count_total_elements(file_content: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let mut reader = Reader::from_str(file_content);
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(8 * 1024);
    let mut count = 0;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"entry" => {
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

/// Represents a historical entry for a game, including the systems it is associated with and various text sections.
#[derive(Debug)]
struct HistoryEntry {
    /// A list of system names associated with the entry.
    names: Vec<String>,
    /// A list of sections containing different types of information (e.g., description, trivia) about the entry.
    sections: Vec<HistorySection>,
}

impl HistoryEntry {
    /// This function initializes a `HistoryEntry` with empty vectors for `names` and `sections`.
    pub fn new() -> Self {
        HistoryEntry {
            names: Vec::new(),
            sections: Vec::new(),
        }
    }
}
