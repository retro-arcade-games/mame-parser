use crate::{
    core::models::collections_helper::{
        get_categories_list, get_languages_list, get_manufacturers_list, get_players_list,
        get_series_list, get_subcategories_list,
    },
    helpers::callback_progress_helper::get_progress_info,
    models::Machine,
    progress::{CallbackType, ProgressCallback, ProgressInfo},
};
use serde_json::{json, to_writer_pretty};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufWriter, Write},
};

/// Writes machine data to multiple JSON files for export.
///
/// This function exports the contents of a `HashMap` of `Machine` data to several JSON files.
/// The main machine data is exported to a primary JSON file, while additional collections such as manufacturers, series, languages, players, categories, and subcategories are exported to separate JSON files.
/// Progress updates are provided through a callback function.
///
/// # Parameters
/// - `export_path`: A `&str` representing the directory path where the JSON files will be exported.
/// - `machines`: A reference to a `HashMap<String, Machine>` containing all machine data to be exported.
///   The key is the machine name, and the value is a `Machine` struct with all associated metadata.
/// - `progress_callback`: A callback function of type `ProgressCallback` that provides progress updates during the JSON writing process.
///   The callback receives a `ProgressInfo` struct containing fields like `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<(), Box<dyn Error + Send + Sync>>`:
/// - On success: Returns `Ok(())` after successfully writing all JSON files to the specified `export_path`.
/// - On failure: Returns an error if there are issues creating or writing to the JSON files.
///
/// # Errors
/// This function will return an error if:
/// - The `machines` HashMap is empty, indicating that there is no data to write.
/// - There are any I/O errors when creating or writing to the JSON files.
/// - The progress callback fails to execute correctly during any phase of the writing process.
///
/// # JSON Files Created
/// This function creates the following JSON files:
/// - `machines.json`: Contains the main machine data, including metadata like name, source file, manufacturer, etc.
/// - `manufacturers.json`: Contains a list of manufacturers and the machines associated with them.
/// - `series.json`: Contains a list of game series and the machines associated with each series.
/// - `languages.json`: Contains a list of languages and the machines available in each language.
/// - `players.json`: Contains player information and the machines that support each player type.
/// - `categories.json`: Contains a list of game categories and the machines that belong to each category.
/// - `subcategories.json`: Contains subcategory data and the machines that belong to each subcategory.
pub fn write_json(
    export_path: &str,
    machines: &HashMap<String, Machine>,
    progress_callback: ProgressCallback,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // If the machines were not loaded, return an error
    if machines.is_empty() {
        return Err("No machines data loaded, please read the data first.".into());
    }

    let total_elements = machines.len();

    export_machines_to_json(export_path, &machines, &progress_callback)?;

    // Export additional collections to separate JSON files
    progress_callback(get_progress_info("Adding manufacturers"));
    export_collection_to_json(
        get_manufacturers_list(&machines),
        export_path,
        "manufacturers",
        false,
    )?;

    progress_callback(get_progress_info("Adding series"));
    export_collection_to_json(get_series_list(&machines), export_path, "series", false)?;

    progress_callback(get_progress_info("Adding languages"));
    export_collection_to_json(
        get_languages_list(&machines),
        export_path,
        "languages",
        false,
    )?;

    progress_callback(get_progress_info("Adding players"));
    export_collection_to_json(get_players_list(&machines), export_path, "players", false)?;

    progress_callback(get_progress_info("Adding categories"));
    export_collection_to_json(
        get_categories_list(&machines),
        export_path,
        "categories",
        false,
    )?;

    progress_callback(get_progress_info("Adding subcategories"));
    export_collection_to_json(
        get_subcategories_list(&machines),
        export_path,
        "subcategories",
        true,
    )?;

    progress_callback(ProgressInfo {
        progress: total_elements as u64,
        total: total_elements as u64,
        message: format!("Json exported successfully to {}", export_path),
        callback_type: CallbackType::Finish,
    });

    Ok(())
}

/// Exports machine data to a JSON file.
///
/// This function exports the contents of a `HashMap` of `Machine` data to a JSON file named `machines.json`.
/// The machines are sorted by name, and each machine's metadata is formatted into a JSON object.
/// The function uses a buffered writer to optimize file writing and provides progress updates via a callback function.
///
/// # Parameters
/// - `export_path`: A `&str` representing the directory path where the `machines.json` file will be created.
/// - `machines`: A reference to a `HashMap<String, Machine>` containing all machine data to be exported.
///   The key is the machine name, and the value is a `Machine` struct with all associated metadata.
/// - `progress_callback`: A reference to a callback function of type `ProgressCallback` that provides progress updates during the JSON writing process.
///   The callback receives a `ProgressInfo` struct containing fields like `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<(), Box<dyn Error + Send + Sync>>`:
/// - On success: Returns `Ok(())` after successfully writing all machine data to the `machines.json` file.
/// - On failure: Returns an error if there are issues creating or writing to the JSON file.
///
/// # Errors
/// This function will return an error if:
/// - The `machines` HashMap is empty, indicating that there is no data to write.
/// - There are any I/O errors when creating or writing to the `machines.json` file.
/// - The progress callback fails to execute correctly during any phase of the writing process.
///
/// # JSON Structure
/// The `machines.json` file contains an array of JSON objects, where each object represents a machine and includes:
/// - Basic metadata: name, source file, manufacturer, etc.
/// - Associated collections: BIOS sets, ROMs, device references, software, samples, history sections, and resources.
/// - Extended data: additional normalized fields such as name, manufacturer, players, parent status, and year.
fn export_machines_to_json(
    export_path: &str,
    machines: &HashMap<String, Machine>,
    progress_callback: &ProgressCallback,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    progress_callback(get_progress_info("Writing machines to JSON"));

    let mut machine_names: Vec<&String> = machines.keys().collect();
    machine_names.sort_unstable();

    let file = File::create(format!("{}/machines.json", export_path))?;
    let mut writer = BufWriter::new(file);
    writer.write_all(b"[\n")?;

    let total_elements = machines.len();
    let batch = std::cmp::max(total_elements / 30, 1);

    for (i, &name) in machine_names.iter().enumerate() {
        let machine = machines.get(name).unwrap(); // Get the machine by name

        if i > 0 {
            writer.write_all(b",\n")?;
        }

        to_writer_pretty(
            &mut writer,
            &json!({
                "name": machine.name,
                "source_file": machine.source_file,
                "rom_of": machine.rom_of,
                "clone_of": machine.clone_of,
                "is_bios": machine.is_bios,
                "is_device": machine.is_device,
                "runnable": machine.runnable,
                "is_mechanical": machine.is_mechanical,
                "sample_of": machine.sample_of,
                "description": machine.description,
                "year": machine.year,
                "manufacturer": machine.manufacturer,
                "bios_sets": machine.bios_sets.iter().map(|bs| json!({
                    "name": bs.name,
                    "description": bs.description,
                })).collect::<Vec<_>>(),
                "roms": machine.roms.iter().map(|rom| json!({
                    "name": rom.name,
                    "size": rom.size,
                    "merge": rom.merge,
                    "status": rom.status,
                    "crc": rom.crc,
                    "sha1": rom.sha1,
                })).collect::<Vec<_>>(),
                "device_refs": machine.device_refs.iter().map(|dr| dr.name.clone()).collect::<Vec<_>>(),
                "software_list": machine.software_list.iter().map(|sw| sw.name.clone()).collect::<Vec<_>>(),
                "samples": machine.samples.iter().map(|sample| sample.name.clone()).collect::<Vec<_>>(),
                "driver_status": machine.driver_status,
                "languages": machine.languages,
                "players": machine.players,
                "series": machine.series,
                "category": machine.category,
                "subcategory": machine.subcategory,
                "is_mature": machine.is_mature,
                "history_sections": machine.history_sections.iter().map(|hs| json!({
                    "order": hs.order,
                    "name": hs.name,
                    "text": hs.text,
                })).collect::<Vec<_>>(),
                "disks": machine.disks.iter().map(|disk| json!({
                    "name": disk.name,
                    "sha1": disk.sha1,
                    "merge": disk.merge,
                    "status": disk.status,
                    "region": disk.region,
                })).collect::<Vec<_>>(),
                "extended_data": machine.extended_data.as_ref().map(|ext| json!({
                    "name": ext.name,
                    "manufacturer": ext.manufacturer,
                    "players": ext.players.as_deref().unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>(),
                    "is_parent": ext.is_parent,
                    "year": ext.year,
                })),
                "resources": machine.resources.iter().map(|res| json!({
                    "type_": res.type_,
                    "name": res.name,
                    "size": res.size,
                    "crc": res.crc,
                    "sha1": res.sha1,
                })).collect::<Vec<_>>(),
            }),
        )?;

        // Progress callback
        if (i + 1) % batch == 0 {
            progress_callback(ProgressInfo {
                progress: (i + 1) as u64,
                total: total_elements as u64,
                message: String::from(""),
                callback_type: CallbackType::Progress,
            });
        }
    }

    writer.write_all(b"\n]")?;
    writer.flush()?;

    Ok(())
}

/// Creates a file for writing JSON data.
///
/// This function creates a file with the specified name in the given export path, which will be used for writing JSON data.
/// If the file does not exist, it will be created; if it does exist, its contents will be overwritten.
///
/// # Parameters
/// - `export_path`: A `&str` representing the directory path where the JSON file should be created.
/// - `file_name`: A `&str` representing the base name of the JSON file (without extension).
///
/// # Returns
/// Returns a `Result<File, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `File` object that can be used to write JSON data.
/// - On failure: Contains an error if the file cannot be created or there are issues with file access permissions.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be created due to permission issues or if the path is invalid.
/// - There are I/O errors while creating the file.
fn create_json_writer(
    export_path: &str,
    file_name: &str,
) -> Result<File, Box<dyn Error + Send + Sync>> {
    let file_path = format!("{}/{}.json", export_path, file_name);
    let file = File::create(file_path)?;
    Ok(file)
}

/// Exports a collection of data to a JSON file.
///
/// This function exports a `HashMap` containing data entries and their associated counts to a JSON file.
/// The data can represent categories or subcategories, depending on the `is_subcategory` flag.
/// The JSON file is created in the specified export path, and the data is written in a formatted JSON structure.
///
/// # Parameters
/// - `data`: A `HashMap<String, usize>` where the key represents the name (category or subcategory), and the value is the count associated with that name.
/// - `export_path`: A `&str` representing the directory path where the JSON file will be created.
/// - `file_name`: A `&str` representing the base name of the JSON file (without extension).
/// - `is_subcategory`: A `bool` indicating whether the data represents subcategories (`true`) or categories (`false`).
///
/// # Returns
/// Returns a `Result<(), Box<dyn Error + Send + Sync>>`:
/// - On success: Returns `Ok(())` after successfully writing all data to the JSON file.
/// - On failure: Returns an error if there are issues creating or writing to the JSON file.
///
/// # Errors
/// This function will return an error if:
/// - The JSON file cannot be created due to permission issues or an invalid path.
/// - There are I/O errors while writing to the JSON file.
/// - The data is improperly formatted or cannot be split correctly when `is_subcategory` is `true`.
///
/// # JSON Structure
/// The JSON file contains an array of JSON objects:
/// - If `is_subcategory` is `true`, each object includes a "category", "subcategory", and the associated "machines" count.
/// - If `is_subcategory` is `false`, each object includes a "name" and the associated "machines" count.
fn export_collection_to_json(
    data: HashMap<String, usize>,
    export_path: &str,
    file_name: &str,
    is_subcategory: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut data_vec: Vec<(&String, &usize)> = data.iter().collect();
    data_vec.sort_by_key(|&(name, _)| name);

    let mut wtr = create_json_writer(export_path, file_name)?;
    let json_data: Vec<_>;
    // Convert the data to a vector of JSON objects
    if is_subcategory {
        json_data = data_vec
            .into_iter()
            .map(|(name, machines)| {
                let splitted: Vec<&str> = name.split(" - ").collect();
                let category = splitted[0];
                let subcategory = splitted[1];
                json!({
                    "category": category,
                    "subcategory": subcategory,
                    "machines": machines,
                })
            })
            .collect();
    } else {
        json_data = data_vec
            .into_iter()
            .map(|(name, machines)| {
                json!({
                    "name": name,
                    "machines": machines,
                })
            })
            .collect();
    }

    // Write the data
    serde_json::to_writer_pretty(&mut wtr, &json_data)?;
    wtr.flush()?;

    Ok(())
}
