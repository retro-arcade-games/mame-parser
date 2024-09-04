use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::models::{HistorySection, Machine};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;

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

    let mut current_entry: Option<HistoryEntry> = None;

    let mut processed_count = 0;
    let batch = total_elements / 10;

    loop {
        match xml_reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                process_node(e, &mut xml_reader, &mut current_entry)?;
            }
            Ok(Event::Empty(ref e)) => {
                process_node(e, &mut xml_reader, &mut current_entry)?;
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
    current_entry: &mut Option<HistoryEntry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match e.name() {
        b"entry" => {
            let entry = HistoryEntry {
                names: Vec::new(),
                sections: Vec::new(),
            };
            *current_entry = Some(entry);
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
            if let Some(ref mut entry) = current_entry {
                entry.names.push(system_name.clone());
            }
        }
        b"text" => {
            let text = reader.read_text(b"text", &mut Vec::new())?;
            let sections = parse_text(&text);
            if let Some(ref mut entry) = current_entry {
                entry.sections = sections;
            }
        }
        _ => (),
    }

    Ok(())
}

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

pub fn count_total_elements(file_content: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
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

#[derive(Debug)]
struct HistoryEntry {
    names: Vec<String>,
    sections: Vec<HistorySection>,
}
