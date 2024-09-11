use crate::{
    core::models::collections_helper::{
        get_categories_list, get_languages_list, get_manufacturers_list, get_players_list,
        get_series_list, get_subcategories_list,
    },
    helpers::callback_progress_helper::get_progress_info,
    models::Machine,
    progress::{CallbackType, ProgressCallback, ProgressInfo},
};
use csv::Writer;
use std::{collections::HashMap, error::Error, fs::File, io::Write};

/// Writes machine data to multiple CSV files for export.
///
/// This function writes the contents of a `HashMap` of `Machine` data to several CSV files,
/// each representing different categories of information such as machines, ROMs, BIOS sets, device references, disks, software, samples, history sections, and resources.
/// The data is exported to the specified path, and progress updates are provided through a callback function.
///
/// # Parameters
/// - `export_path`: A `&str` representing the path where the CSV files will be exported.
/// - `machines`: A reference to a `HashMap<String, Machine>` containing all machine data to be exported.
///   The key is the machine name, and the value is a `Machine` struct with all associated metadata.
/// - `progress_callback`: A callback function of type `ProgressCallback` that provides progress updates during the CSV writing process.
///   The callback receives a `ProgressInfo` struct containing fields like `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<(), Box<dyn Error + Send + Sync>>`:
/// - On success: Returns `Ok(())` after successfully writing all CSV files to the specified `export_path`.
/// - On failure: Returns an error if there are issues creating or writing to the CSV files.
///
/// # Errors
/// This function will return an error if:
/// - The `machines` HashMap is empty, indicating that there is no data to write.
/// - There are any I/O errors when creating or writing to the CSV files.
/// - The progress callback fails to execute correctly during any phase of the writing process.
///
/// # CSV Files Created
/// This function creates the following CSV files:
/// - `machines.csv`: Contains the main machine data, including metadata like name, source file, manufacturer, etc.
/// - `roms.csv`: Contains ROM-specific data for each machine.
/// - `bios_sets.csv`: Contains BIOS set information linked to each machine.
/// - `device_refs.csv`: Contains device reference data linked to each machine.
/// - `disks.csv`: Contains disk information for each machine.
/// - `softwares.csv`: Contains software information linked to each machine.
/// - `samples.csv`: Contains sample data for each machine.
/// - `history_sections.csv`: Contains historical information and sections for each machine.
/// - `resources.csv`: Contains resource information such as size, type, and checksums for each machine.
/// - `manufacturers.csv`: Contains a list of manufacturers and the machines associated with them.
/// - `series.csv`: Contains a list of game series and the machines associated with each series.
/// - `languages.csv`: Contains a list of languages and the machines available in each language.
/// - `players.csv`: Contains player information and the machines that support each player type.
/// - `categories.csv`: Contains a list of game categories and the machines that belong to each category.
/// - `subcategories.csv`: Contains subcategory data and the machines that belong to each subcategory.
///
pub fn write_csv(
    export_path: &str,
    machines: &HashMap<String, Machine>,
    progress_callback: ProgressCallback,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // If the machines were not loaded, return an error
    if machines.is_empty() {
        return Err("No machines data loaded, please read the data first.".into());
    }

    progress_callback(get_progress_info(
        format!("Writing CSV files to {}", export_path).as_str(),
    ));

    let total_elements = machines.len();
    let mut processed_count = 0;
    let batch = total_elements / 10;

    let mut machines_vec: Vec<(&String, &Machine)> = machines.iter().collect();
    machines_vec.sort_by_key(|&(name, _)| name);

    // Create the CSV writers
    let mut machines_wtr = create_writer(export_path, "machines")?;
    let mut roms_wtr = create_writer(export_path, "roms")?;
    let mut bios_sets_wtr = create_writer(export_path, "bios_sets")?;
    let mut device_refs_wtr = create_writer(export_path, "device_refs")?;
    let mut disks_wtr = create_writer(export_path, "disks")?;
    let mut softwares_wtr = create_writer(export_path, "softwares")?;
    let mut samples_wtr = create_writer(export_path, "samples")?;
    let mut history_sections_wtr = create_writer(export_path, "history_sections")?;
    let mut resources_wtr = create_writer(export_path, "resources")?;

    // Write the CSV headers
    write_csv_header(
        &mut machines_wtr,
        &[
            "name",
            "source_file",
            "rom_of",
            "clone_of",
            "is_bios",
            "is_device",
            "runnable",
            "is_mechanical",
            "sample_of",
            "description",
            "year",
            "manufacturer",
            "driver_status",
            "languages",
            "players",
            "series",
            "category",
            "subcategory",
            "is_mature",
            "extended_name",
            "extended_manufacturer",
            "extended_players",
            "extended_is_parent",
            "extended_year",
        ],
    )?;
    write_csv_header(
        &mut roms_wtr,
        &[
            "machine_name",
            "name",
            "size",
            "merge",
            "status",
            "crc",
            "sha1",
        ],
    )?;
    write_csv_header(&mut bios_sets_wtr, &["machine_name", "name", "description"])?;
    write_csv_header(&mut device_refs_wtr, &["machine_name", "name"])?;
    write_csv_header(
        &mut disks_wtr,
        &["machine_name", "name", "sha1", "merge", "status", "region"],
    )?;
    write_csv_header(&mut softwares_wtr, &["machine_name", "name"])?;
    write_csv_header(&mut samples_wtr, &["machine_name", "name"])?;
    write_csv_header(
        &mut history_sections_wtr,
        &["machine_name", "name", "text", "order"],
    )?;
    write_csv_header(
        &mut resources_wtr,
        &["machine_name", "type", "name", "size", "crc", "sha1"],
    )?;

    for (name, machine) in machines_vec {
        // Write machine
        write_csv_record(
            &mut machines_wtr,
            &[
                name,
                machine.source_file.as_deref().unwrap_or(""),
                machine.rom_of.as_deref().unwrap_or(""),
                machine.clone_of.as_deref().unwrap_or(""),
                machine
                    .is_bios
                    .map(|is_bios| if is_bios { "true" } else { "false" })
                    .unwrap_or(""),
                machine
                    .is_device
                    .map(|is_device| if is_device { "true" } else { "false" })
                    .unwrap_or(""),
                machine
                    .runnable
                    .map(|runnable| if runnable { "true" } else { "false" })
                    .unwrap_or(""),
                machine
                    .is_mechanical
                    .map(|is_mechanical| if is_mechanical { "true" } else { "false" })
                    .unwrap_or(""),
                machine.sample_of.as_deref().unwrap_or(""),
                machine.description.as_deref().unwrap_or(""),
                machine.year.as_deref().unwrap_or(""),
                machine.manufacturer.as_deref().unwrap_or(""),
                machine.driver_status.as_deref().unwrap_or(""),
                &machine.languages.join(", "),
                machine.players.as_deref().unwrap_or(""),
                machine.series.as_deref().unwrap_or(""),
                machine.category.as_deref().unwrap_or(""),
                machine.subcategory.as_deref().unwrap_or(""),
                machine
                    .is_mature
                    .map(|is_mature| if is_mature { "true" } else { "false" })
                    .unwrap_or(""),
                machine
                    .extended_data
                    .as_ref()
                    .unwrap()
                    .name
                    .as_deref()
                    .unwrap_or(""),
                machine
                    .extended_data
                    .as_ref()
                    .unwrap()
                    .manufacturer
                    .as_deref()
                    .unwrap_or(""),
                machine
                    .extended_data
                    .as_ref()
                    .unwrap()
                    .players
                    .as_deref()
                    .unwrap_or(""),
                machine
                    .extended_data
                    .as_ref()
                    .unwrap()
                    .is_parent
                    .map(|is_parent| if is_parent { "true" } else { "false" })
                    .unwrap_or(""),
                machine
                    .extended_data
                    .as_ref()
                    .unwrap()
                    .year
                    .as_deref()
                    .unwrap_or(""),
            ],
        )?;
        // Write roms
        for rom in &machine.roms {
            write_csv_record(
                &mut roms_wtr,
                &[
                    name,
                    &rom.name,
                    &rom.size.to_string(),
                    rom.merge.as_deref().unwrap_or(""),
                    rom.status.as_deref().unwrap_or(""),
                    rom.crc.as_deref().unwrap_or(""),
                    rom.sha1.as_deref().unwrap_or(""),
                ],
            )?;
        }
        // Write bios sets
        for bios_set in &machine.bios_sets {
            write_csv_record(
                &mut bios_sets_wtr,
                &[name, &bios_set.name, &bios_set.description],
            )?;
        }
        // Write device refs
        for device_ref in &machine.device_refs {
            write_csv_record(&mut device_refs_wtr, &[name, &device_ref.name])?;
        }
        // Write disks
        for disk in &machine.disks {
            write_csv_record(
                &mut disks_wtr,
                &[
                    name,
                    &disk.name,
                    disk.sha1.as_deref().unwrap_or(""),
                    disk.merge.as_deref().unwrap_or(""),
                    disk.status.as_deref().unwrap_or(""),
                    disk.region.as_deref().unwrap_or(""),
                ],
            )?;
        }
        // Write softwares
        for software in &machine.software_list {
            write_csv_record(&mut softwares_wtr, &[name, &software.name])?;
        }
        // Write samples
        for sample in &machine.samples {
            write_csv_record(&mut samples_wtr, &[name, &sample.name])?;
        }
        // Write history sections
        for history_section in &machine.history_sections {
            write_csv_record(
                &mut history_sections_wtr,
                &[
                    name,
                    &history_section.name,
                    &history_section.text,
                    &history_section.order.to_string(),
                ],
            )?;
        }
        // Write resources
        for resource in &machine.resources {
            write_csv_record(
                &mut resources_wtr,
                &[
                    name,
                    &resource.type_,
                    &resource.name,
                    &resource.size.to_string(),
                    &resource.crc,
                    &resource.sha1,
                ],
            )?;
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

    machines_wtr.flush()?;
    roms_wtr.flush()?;
    bios_sets_wtr.flush()?;
    device_refs_wtr.flush()?;
    disks_wtr.flush()?;
    softwares_wtr.flush()?;
    samples_wtr.flush()?;
    history_sections_wtr.flush()?;
    resources_wtr.flush()?;

    progress_callback(get_progress_info("Adding manufacturers"));
    export_collection(
        get_manufacturers_list(&machines),
        export_path,
        "manufacturers",
        &["name", "machines"],
        false,
    )?;

    progress_callback(get_progress_info("Adding series"));
    export_collection(
        get_series_list(&machines),
        export_path,
        "series",
        &["name", "machines"],
        false,
    )?;

    progress_callback(get_progress_info("Adding languages"));
    export_collection(
        get_languages_list(&machines),
        export_path,
        "languages",
        &["name", "machines"],
        false,
    )?;

    progress_callback(get_progress_info("Adding players"));
    export_collection(
        get_players_list(&machines),
        export_path,
        "players",
        &["name", "machines"],
        false,
    )?;

    progress_callback(get_progress_info("Adding categories"));
    export_collection(
        get_categories_list(&machines),
        export_path,
        "categories",
        &["name", "machines"],
        false,
    )?;

    progress_callback(get_progress_info("Adding subcategories"));
    export_collection(
        get_subcategories_list(&machines),
        export_path,
        "subcategories",
        &["category", "subcategory", "machines"],
        true,
    )?;

    progress_callback(ProgressInfo {
        progress: processed_count as u64,
        total: processed_count as u64,
        message: format!("CSVs exported successfully to {}", export_path),
        callback_type: CallbackType::Finish,
    });

    Ok(())
}

/// Creates a CSV writer for a specific file.
///
/// This function creates a CSV writer for a file with the specified name, located in the given export path.
/// The writer is set up to handle outputting data in CSV format to the created file.
/// If the file does not exist, it will be created; if it does exist, its contents will be overwritten.
///
/// # Parameters
/// - `export_path`: A `&str` representing the directory path where the CSV file should be created.
/// - `file_name`: A `&str` representing the base name of the CSV file (without extension) to be created.
///
/// # Returns
/// Returns a `Result<Writer<File>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `Writer<File>` that can be used to write data to the specified CSV file.
/// - On failure: Contains an error if the file cannot be created or there are issues with file access permissions.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be created due to permission issues or if the path is invalid.
/// - There are I/O errors while creating the file or initializing the writer.
fn create_writer(
    export_path: &str,
    file_name: &str,
) -> Result<Writer<File>, Box<dyn Error + Send + Sync>> {
    let file_path = format!("{}/{}.csv", export_path, file_name);
    let file = File::create(file_path)?;
    let writer = Writer::from_writer(file);
    Ok(writer)
}

/// Writes a header row to a CSV file.
///
/// This function writes the provided header fields to the beginning of a CSV file using the given CSV writer.
/// The headers define the columns of the CSV file, providing structure to the data that follows.
///
/// # Parameters
/// - `wtr`: A mutable reference to a `Writer<File>` representing the CSV writer where the headers will be written.
/// - `headers`: A slice of `&str` containing the header fields to be written to the CSV file.
///
/// # Returns
/// Returns a `Result<(), csv::Error>`:
/// - On success: Returns `Ok(())` after successfully writing the header row.
/// - On failure: Returns a `csv::Error` if there is an issue writing the header to the CSV file.
///
/// # Errors
/// This function will return an error if:
/// - There are I/O issues while writing to the CSV file.
/// - The CSV writer encounters an internal error while processing the headers.
fn write_csv_header(wtr: &mut Writer<File>, headers: &[&str]) -> Result<(), csv::Error> {
    wtr.write_record(headers)
}

/// Writes a data record to a CSV file.
///
/// This function writes a row of data fields to a CSV file using the provided CSV writer.
/// Each field corresponds to a column in the CSV file, maintaining the order defined by the headers.
///
/// # Parameters
/// - `wtr`: A mutable reference to a `Writer<W>` where `W` implements `Write`. This represents the CSV writer that will be used to write the record.
/// - `fields`: A slice of `&str` containing the data fields to be written as a single row in the CSV file.
///
/// # Returns
/// Returns a `Result<(), csv::Error>`:
/// - On success: Returns `Ok(())` after successfully writing the data record.
/// - On failure: Returns a `csv::Error` if there is an issue writing the data record to the CSV file.
///
/// # Errors
/// This function will return an error if:
/// - There are I/O issues while writing to the CSV file.
/// - The CSV writer encounters an internal error while processing the data record.
fn write_csv_record<W: Write>(wtr: &mut Writer<W>, fields: &[&str]) -> Result<(), csv::Error> {
    wtr.write_record(fields)
}

/// Exports a collection of data to a CSV file.
///
/// This function exports a `HashMap` containing data entries and their associated counts to a CSV file.
/// The data can represent categories or subcategories, depending on the `is_subcategory` flag.
/// The file is created in the specified export path, and headers are written before the data rows.
///
/// # Parameters
/// - `data`: A `HashMap<String, usize>` where the key represents the name (category or subcategory), and the value is the count associated with that name.
/// - `export_path`: A `&str` representing the directory path where the CSV file will be created.
/// - `file_name`: A `&str` representing the base name of the CSV file (without extension).
/// - `headers`: A slice of `&str` containing the header fields to be written to the CSV file.
/// - `is_subcategory`: A `bool` indicating whether the data represents subcategories (`true`) or categories (`false`).
///
/// # Returns
/// Returns a `Result<(), Box<dyn Error + Send + Sync>>`:
/// - On success: Returns `Ok(())` after successfully writing all data to the CSV file.
/// - On failure: Returns an error if there are issues creating or writing to the CSV file.
///
/// # Errors
/// This function will return an error if:
/// - The CSV file cannot be created due to permission issues or an invalid path.
/// - There are I/O errors while writing to the CSV file.
/// - The data is improperly formatted or cannot be split correctly when `is_subcategory` is `true`.
fn export_collection(
    data: HashMap<String, usize>,
    export_path: &str,
    file_name: &str,
    headers: &[&str],
    is_subcategory: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut data_vec: Vec<(&String, &usize)> = data.iter().collect();
    data_vec.sort_by_key(|&(name, _)| name);

    // Create the file path
    let file_path = format!("{}/{}.csv", export_path, file_name);
    let file = File::create(file_path)?;
    let mut wtr = Writer::from_writer(file);

    // Write the header
    wtr.write_record(headers)?;

    match is_subcategory {
        true => {
            for (name, count) in data_vec {
                let splitted: Vec<&str> = name.split(" - ").collect();
                let category = splitted[0];
                let subcategory = splitted[1];
                wtr.write_record(&[category, subcategory, &count.to_string()])?;
            }
        }
        false => {
            for (name, count) in data_vec {
                wtr.write_record(&[name, &count.to_string()])?;
            }
        }
    }

    wtr.flush()?;

    Ok(())
}
