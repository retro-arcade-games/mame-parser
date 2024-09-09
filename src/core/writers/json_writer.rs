use crate::{
    models::Machine,
    progress::{CallbackType, ProgressCallback, ProgressInfo},
};
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::Write,
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

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding manufacturers"),
        callback_type: CallbackType::Info,
    });

    let mut manufacturers: Vec<String> = machines
        .values()
        .filter_map(|machine| machine.manufacturer.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    manufacturers.sort_unstable();
    export_collection_to_json(manufacturers, export_path, "manufacturers")?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding series"),
        callback_type: CallbackType::Info,
    });

    let mut series: Vec<String> = machines
        .values()
        .filter_map(|machine| machine.series.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    series.sort_unstable();
    export_collection_to_json(series, export_path, "series")?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding languages"),
        callback_type: CallbackType::Info,
    });

    let mut languages: Vec<String> = machines
        .values()
        .flat_map(|machine| machine.languages.iter().cloned())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    languages.sort_unstable();
    export_collection_to_json(languages, export_path, "languages")?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding players"),
        callback_type: CallbackType::Info,
    });

    let mut players: Vec<String> = machines
        .values()
        .filter_map(|machine| machine.extended_data.as_ref()?.players.as_ref())
        .flat_map(|players| players.split(',').map(|s| s.trim().to_string()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    players.sort_unstable();
    export_collection_to_json(players, export_path, "players")?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding categories"),
        callback_type: CallbackType::Info,
    });

    let mut categories: Vec<String> = machines
        .values()
        .filter_map(|machine| machine.category.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    categories.sort_unstable();
    export_collection_to_json(categories, export_path, "categories")?;

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Adding subcategories"),
        callback_type: CallbackType::Info,
    });

    export_subcategories_to_json(export_path, &machines)?;

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
    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Loading machines information"),
        callback_type: CallbackType::Info,
    });

    // Get the machines data
    let mut machines_vec: Vec<(&String, &Machine)> = machines.iter().collect();
    machines_vec.sort_by_key(|&(name, _)| name);

    // Create the JSON writer for machines.json
    let mut machines_wtr = create_json_writer(export_path, "machines")?;

    // Collect all machines into a vector for JSON with transformations
    let machines_json: Vec<_> = machines_vec.into_iter().map(|(_, machine)| {
    json!({
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
    })
}).collect::<Vec<_>>();

    let total_elements = machines_json.len();
    let mut processed_count = 0;
    let batch = total_elements / 30;

    // Write the opening of the array
    machines_wtr.write_all(b"[\n")?;

    for (i, item) in machines_json.iter().enumerate() {
        if i > 0 {
            machines_wtr.write_all(b",\n")?;
        }
        serde_json::to_writer_pretty(&mut machines_wtr, &item)?;

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

    // Write the closing of the array
    machines_wtr.write_all(b"\n]")?;

    machines_wtr.flush()?;

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
    data: Vec<String>,
    export_path: &str,
    file_name: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut wtr = create_json_writer(export_path, file_name)?;

    // Convert the data to a vector of JSON objects
    let json_data: Vec<_> = data
        .into_iter()
        .map(|name| {
            json!({
                "name": name,
            })
        })
        .collect();

    // Write the data
    serde_json::to_writer_pretty(&mut wtr, &json_data)?;
    wtr.flush()?;

    Ok(())
}

fn export_subcategories_to_json(
    export_path: &str,
    machines: &HashMap<String, Machine>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut subcategories: Vec<String> = machines
        .values()
        .filter_map(|machine| {
            machine.category.as_ref().and_then(|category| {
                machine
                    .subcategory
                    .as_ref()
                    .map(|subcategory| format!("{} - {}", category, subcategory))
            })
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    subcategories.sort_unstable();

    let mut subcategories_wtr = create_json_writer(export_path, "subcategories")?;

    let json_data: Vec<_> = subcategories
        .into_iter()
        .map(|name| {
            let splitted: Vec<&str> = name.split(" - ").collect();
            let category = splitted[0];
            let subcategory = splitted[1];
            json!({
                "category": category,
                "subcategory": subcategory,
            })
        })
        .collect();

    // Write the subcategories data
    serde_json::to_writer_pretty(&mut subcategories_wtr, &json_data)?;
    subcategories_wtr.flush()?;

    Ok(())
}
