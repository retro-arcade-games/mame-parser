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

fn create_json_writer(
    export_path: &str,
    file_name: &str,
) -> Result<File, Box<dyn Error + Send + Sync>> {
    let file_path = format!("{}/{}.json", export_path, file_name);
    let file = File::create(file_path)?;
    Ok(file)
}

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
