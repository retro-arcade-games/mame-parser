use crate::{
    core::models::collections_helper::{
        get_categories_list, get_languages_list, get_manufacturers_list, get_players_list,
        get_series_list, get_subcategories_list,
    },
    models::Machine,
    progress::{CallbackType, ProgressCallback, ProgressInfo},
};
use csv::Writer;
use std::{collections::HashMap, error::Error, fs::File, io::Write};

pub fn write_csv(
    export_path: &str,
    machines: &HashMap<String, Machine>,
    progress_callback: ProgressCallback,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // If the machines were not loaded, return an error
    if machines.is_empty() {
        return Err("No machines data loaded, please read the data first.".into());
    }

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Writing csv files to {}", export_path),
        callback_type: CallbackType::Info,
    });

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

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding manufacturers"),
        callback_type: CallbackType::Info,
    });

    export_collection(
        get_manufacturers_list(&machines),
        export_path,
        "manufacturers",
        &["name", "machines"],
        false,
    )?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding series"),
        callback_type: CallbackType::Info,
    });

    export_collection(
        get_series_list(&machines),
        export_path,
        "series",
        &["name", "machines"],
        false,
    )?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding languages"),
        callback_type: CallbackType::Info,
    });

    export_collection(
        get_languages_list(&machines),
        export_path,
        "languages",
        &["name", "machines"],
        false,
    )?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding players"),
        callback_type: CallbackType::Info,
    });

    export_collection(
        get_players_list(&machines),
        export_path,
        "players",
        &["name", "machines"],
        false,
    )?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding categories"),
        callback_type: CallbackType::Info,
    });

    export_collection(
        get_categories_list(&machines),
        export_path,
        "categories",
        &["name", "machines"],
        false,
    )?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding subcategories"),
        callback_type: CallbackType::Info,
    });

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

fn create_writer(
    export_path: &str,
    file_name: &str,
) -> Result<Writer<File>, Box<dyn Error + Send + Sync>> {
    let file_path = format!("{}/{}.csv", export_path, file_name);
    let file = File::create(file_path)?;
    let writer = Writer::from_writer(file);
    Ok(writer)
}

fn write_csv_header(wtr: &mut Writer<File>, headers: &[&str]) -> Result<(), csv::Error> {
    wtr.write_record(headers)
}

fn write_csv_record<W: Write>(wtr: &mut Writer<W>, fields: &[&str]) -> Result<(), csv::Error> {
    wtr.write_record(fields)
}

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
