use crate::core::models::collections_helper::{
    get_languages_list, get_manufacturers_list, get_players_list, get_series_list,
};
use crate::helpers::callback_progress_helper::get_progress_info;
use crate::models::Machine;
use crate::progress::{CallbackType, ProgressCallback, ProgressInfo};
use rusqlite::{params, Connection, Result, Transaction};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

pub fn write_sqlite(
    data_base_path: &str,
    machines: &HashMap<String, Machine>,
    progress_callback: ProgressCallback,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // If the machines were not loaded, return an error
    if machines.is_empty() {
        return Err("No machines data loaded, please read the data first.".into());
    }

    // Remove the database file if it already exists
    if fs::metadata(data_base_path).is_ok() {
        let _ = fs::remove_file(data_base_path);
    }

    let mut conn = Connection::open(data_base_path).unwrap();

    create_database(&mut conn)?;

    let batch_size = 5000;
    let mut batch_count = 0;

    let total_elements = machines.len();

    progress_callback(get_progress_info(
        format!("Writing {}", data_base_path).as_str(),
    ));
    let mut processed_count = 0;
    let batch = 5000;

    let mut transaction = conn.transaction()?;
    for machine in machines.values() {
        insert_machine_data(&transaction, machine)?;

        batch_count += 1;
        if batch_count >= batch_size {
            transaction.commit()?;
            transaction = conn.transaction()?;
            batch_count = 0;
        }

        processed_count += 1;
        if processed_count % batch == 0 {
            progress_callback(ProgressInfo {
                progress: processed_count as u64,
                total: total_elements as u64,
                message: String::from(""),
                callback_type: CallbackType::Progress,
            });
        }
    }

    // Commit any remaining transactions
    transaction.commit()?;

    progress_callback(ProgressInfo {
        progress: processed_count as u64,
        total: total_elements as u64,
        message: String::from(""),
        callback_type: CallbackType::Progress,
    });

    // Add relations
    create_relations(&mut conn, &machines, &progress_callback)?;

    // Add languages relations
    progress_callback(get_progress_info("Adding languages relations"));
    extract_and_insert_languages(&mut conn, &machines)?;
    insert_machine_language_relationships(&mut conn)?;

    // Add players relations
    progress_callback(get_progress_info("Adding players relations"));
    extract_and_insert_players(&mut conn, &machines)?;
    insert_machine_player_relationships(&mut conn)?;

    let data_base_file = data_base_path.split('/').last().unwrap();
    progress_callback(ProgressInfo {
        progress: processed_count as u64,
        total: processed_count as u64,
        message: format!("{} exported successfully", data_base_file),
        callback_type: CallbackType::Finish,
    });

    Ok(())
}

/**
 * Create the database and the required tables.
 */
fn create_database(conn: &mut Connection) -> Result<()> {
    // Series table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS series (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             name TEXT NOT NULL UNIQUE
         )",
        [],
    )?;

    // Categories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             name TEXT NOT NULL UNIQUE
         )",
        [],
    )?;

    // Subcategories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS subcategories (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             name TEXT NOT NULL,
             category_id INTEGER,
             UNIQUE(name, category_id),
             FOREIGN KEY (category_id) REFERENCES categories(id)
         )",
        [],
    )?;

    // Manufacturers table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS manufacturers (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             name TEXT NOT NULL UNIQUE
         )",
        [],
    )?;

    // Languages table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS languages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    // Players table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    // Machines table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS machines (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  name TEXT NOT NULL UNIQUE,
                  source_file TEXT,
                  rom_of TEXT,
                  clone_of TEXT,
                  is_bios INTEGER,
                  is_device INTEGER,
                  runnable INTEGER,
                  is_mechanical INTEGER,
                  sample_of TEXT,
                  description TEXT,
                  year TEXT,
                  manufacturer TEXT,
                  driver_status TEXT,
                  players TEXT,
                  series TEXT,
                  category TEXT,
                  subcategory TEXT,
                  is_mature INTEGER,
                  languages TEXT,
                  category_id INTEGER,
                  subcategory_id INTEGER,
                  series_id INTEGER,
                  manufacturer_id INTEGER,
                  FOREIGN KEY (category_id) REFERENCES categories(id),
                  FOREIGN KEY (subcategory_id) REFERENCES subcategories(id),
                  FOREIGN KEY (series_id) REFERENCES series(id)
                  FOREIGN KEY (manufacturer_id) REFERENCES manufacturers(id)
                  )",
        [],
    )?;

    // Machine languages table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS machine_languages (
            machine_id INTEGER,
            language_id INTEGER,
            FOREIGN KEY(machine_id) REFERENCES machines(id),
            FOREIGN KEY(language_id) REFERENCES languages(id),
            PRIMARY KEY(machine_id, language_id)
        )",
        [],
    )?;

    // Machine players table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS machine_players (
            machine_id INTEGER,
            player_id INTEGER,
            FOREIGN KEY(machine_id) REFERENCES machines(id),
            FOREIGN KEY(player_id) REFERENCES players(id),
            PRIMARY KEY(machine_id, player_id)
        )",
        [],
    )?;

    // Extended data table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS extended_data (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  manufacturer TEXT,
                  players TEXT,
                  is_parent INTEGER,
                  year TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // BIOS sets table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bios_sets (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  description TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // ROMs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS roms (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  size INTEGER,
                  merge TEXT,
                  status TEXT,
                  crc TEXT,
                  sha1 TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // Device refs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS device_refs (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // Softwares table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS softwares (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // Samples table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS samples (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // Disks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS disks (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  sha1 TEXT,
                  merge TEXT,
                  status TEXT,
                  region TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // History sections table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history_sections (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  name TEXT,
                  text TEXT,
                  `order` INTEGER,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    // Resources table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS resources (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  machine_name TEXT,
                  type TEXT,
                  name TEXT,
                  size INTEGER,
                  crc TEXT,
                  sha1 TEXT,
                  machine_id INTEGER,
                  FOREIGN KEY(machine_id) REFERENCES machines(id)
                  )",
        [],
    )?;

    Ok(())
}

fn insert_machine_data(transaction: &Transaction, machine: &Machine) -> Result<()> {
    transaction.execute(
        "INSERT OR REPLACE INTO machines (
                  name, source_file, rom_of, clone_of, is_bios, is_device, runnable, is_mechanical, sample_of,
                  description, year, manufacturer, driver_status, players, series, category, subcategory, is_mature, languages
                  ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        params![
            machine.name,
            machine.source_file,
            machine.rom_of,
            machine.clone_of,
            machine.is_bios,
            machine.is_device,
            machine.runnable,
            machine.is_mechanical,
            machine.sample_of,
            machine.description,
            machine.year,
            machine.manufacturer,
            machine.driver_status,
            machine.players,
            machine.series,
            machine.category,
            machine.subcategory,
            machine.is_mature,
            machine.languages.join(", ")
        ],
    )?;

    if let Some(extended_data) = &machine.extended_data {
        transaction.execute(
            "INSERT OR REPLACE INTO extended_data (machine_name, name, manufacturer, players, is_parent, year) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![machine.name, extended_data.name, extended_data.manufacturer, extended_data.players, extended_data.is_parent, extended_data.year],
        )?;
    }

    for bios_set in &machine.bios_sets {
        transaction.execute(
            "INSERT OR REPLACE INTO bios_sets (machine_name, name, description) VALUES (?1, ?2, ?3)",
            params![machine.name, bios_set.name, bios_set.description],
        )?;
    }

    for rom in &machine.roms {
        transaction.execute(
            "INSERT OR REPLACE INTO roms (
                      machine_name, name, size, merge, status, crc, sha1
                      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                machine.name,
                rom.name,
                rom.size,
                rom.merge,
                rom.status,
                rom.crc,
                rom.sha1
            ],
        )?;
    }

    for device_ref in &machine.device_refs {
        transaction.execute(
            "INSERT OR REPLACE INTO device_refs (machine_name, name) VALUES (?1, ?2)",
            params![machine.name, device_ref.name],
        )?;
    }

    for software in &machine.software_list {
        transaction.execute(
            "INSERT OR REPLACE INTO softwares (machine_name, name) VALUES (?1, ?2)",
            params![machine.name, software.name],
        )?;
    }

    for sample in &machine.samples {
        transaction.execute(
            "INSERT OR REPLACE INTO samples (machine_name, name) VALUES (?1, ?2)",
            params![machine.name, sample.name],
        )?;
    }

    for disk in &machine.disks {
        transaction.execute(
            "INSERT OR REPLACE INTO disks (
                      machine_name, name, sha1, merge, status, region
                      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                machine.name,
                disk.name,
                disk.sha1,
                disk.merge,
                disk.status,
                disk.region
            ],
        )?;
    }

    for history_section in &machine.history_sections {
        transaction.execute(
            "INSERT OR REPLACE INTO history_sections (
                      machine_name, name, text, `order`
                      ) VALUES (?1, ?2, ?3, ?4)",
            params![
                machine.name,
                history_section.name,
                history_section.text,
                history_section.order
            ],
        )?;
    }

    for resource in &machine.resources {
        transaction.execute(
            "INSERT OR REPLACE INTO resources (
                      machine_name, type, name, size, crc, sha1
                      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                machine.name,
                resource.type_,
                resource.name,
                resource.size,
                resource.crc,
                resource.sha1
            ],
        )?;
    }

    Ok(())
}

fn extract_and_insert_languages(
    conn: &mut Connection,
    machines: &HashMap<String, Machine>,
) -> Result<()> {
    let languages_hash = get_languages_list(&machines);
    let mut languages: Vec<String> = languages_hash.keys().cloned().collect();
    languages.sort();

    let tx = conn.transaction()?;
    {
        let mut insert_stmt = tx.prepare("INSERT OR IGNORE INTO languages (name) VALUES (?)")?;
        for language in languages {
            insert_stmt.execute([&language])?;
        }
    }
    tx.commit()?;

    Ok(())
}

fn insert_machine_language_relationships(conn: &mut Connection) -> Result<()> {
    let machine_languages: Vec<(i64, String)> = {
        let mut stmt = conn.prepare("SELECT id, languages FROM machines")?;
        let machine_languages = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let languages: String = row.get(1)?;
            Ok((id, languages))
        })?;
        machine_languages.collect::<Result<Vec<_>, _>>()?
    };

    let tx = conn.transaction()?;
    {
        let mut insert_stmt = tx.prepare(
            "INSERT INTO machine_languages (machine_id, language_id)
             VALUES (?, (SELECT id FROM languages WHERE name = ?))",
        )?;
        for (machine_id, languages) in machine_languages {
            for language in languages.split(',').map(|s| s.trim()) {
                insert_stmt.execute(params![machine_id, language])?;
            }
        }
    }
    tx.commit()?;

    Ok(())
}

fn extract_and_insert_players(
    conn: &mut Connection,
    machines: &HashMap<String, Machine>,
) -> Result<()> {
    let players_hash = get_players_list(&machines);
    let mut players: Vec<String> = players_hash.keys().cloned().collect();
    players.sort();

    let tx = conn.transaction()?;
    {
        let mut insert_stmt = tx.prepare("INSERT OR IGNORE INTO players (name) VALUES (?)")?;
        for player in players {
            insert_stmt.execute([&player])?;
        }
    }
    tx.commit()?;

    Ok(())
}

fn insert_machine_player_relationships(conn: &mut Connection) -> Result<()> {
    let machine_players: Vec<(i64, String)> = {
        let mut stmt = conn.prepare(
            "SELECT machines.id, extended_data.players
             FROM machines
             INNER JOIN extended_data ON machines.id = extended_data.machine_id
             WHERE extended_data.players IS NOT NULL",
        )?;
        let machine_players = stmt.query_map([], |row| {
            let machine_id: i64 = row.get(0)?;
            let players: String = row.get(1)?;
            Ok((machine_id, players))
        })?;
        machine_players.collect::<Result<Vec<_>, _>>()?
    };

    let tx = conn.transaction()?;
    {
        let mut insert_stmt = tx.prepare(
            "INSERT INTO machine_players (machine_id, player_id)
             VALUES (?, (SELECT id FROM players WHERE name = ?))",
        )?;
        for (machine_id, players) in machine_players {
            for player in players.split(',').map(|s| s.trim()) {
                insert_stmt.execute(params![machine_id, player])?;
            }
        }
    }
    tx.commit()?;

    Ok(())
}

fn create_relations(
    conn: &mut Connection,
    machines: &HashMap<String, Machine>,
    progress_callback: &ProgressCallback,
) -> Result<()> {
    progress_callback(get_progress_info("Creating relations"));
    // Add categories
    conn.execute(
        "INSERT OR IGNORE INTO categories (name)
         SELECT DISTINCT category FROM machines WHERE category IS NOT NULL ORDER BY category",
        [],
    )?;
    // Update machines with category_id
    conn.execute(
        "UPDATE machines
         SET category_id = (SELECT id FROM categories WHERE categories.name = machines.category)",
        [],
    )?;
    // Add subcategories (must be executed after updating machines with category_id)
    conn.execute(
        "INSERT OR IGNORE INTO subcategories (name, category_id)
         SELECT DISTINCT subcategory, category_id
         FROM machines
         WHERE subcategory IS NOT NULL ORDER BY subcategory",
        [],
    )?;
    // Update machines with subcategory_id
    conn.execute(
        "UPDATE machines
         SET subcategory_id = (
             SELECT id
             FROM subcategories
             WHERE subcategories.name = machines.subcategory
               AND subcategories.category_id = machines.category_id
         )",
        [],
    )?;

    progress_callback(get_progress_info("Adding series"));
    // Add series
    let series_hash = get_series_list(&machines);
    let mut series: Vec<String> = series_hash.keys().cloned().collect();
    series.sort();

    // let series = get_list(&SERIES);
    let tx = conn.transaction()?;
    {
        let mut insert_stmt = tx.prepare("INSERT OR IGNORE INTO series (name) VALUES (?)")?;
        for series_name in series {
            insert_stmt.execute([&series_name])?;
        }
    }
    tx.commit()?;
    // Update machines with series_id
    conn.execute(
        "UPDATE machines
         SET series_id = (SELECT id FROM series WHERE series.name = machines.series)",
        [],
    )?;

    progress_callback(get_progress_info("Adding manufacturers"));
    // Add manufacturers from extended data
    let manufacturers_hash = get_manufacturers_list(&machines);
    let mut manufacturers: Vec<String> = manufacturers_hash.keys().cloned().collect();
    manufacturers.sort();
    let tx = conn.transaction()?;
    {
        let mut insert_stmt =
            tx.prepare("INSERT OR IGNORE INTO manufacturers (name) VALUES (?)")?;
        for manufacturer in manufacturers {
            insert_stmt.execute([&manufacturer])?;
        }
    }
    tx.commit()?;

    progress_callback(get_progress_info("Updating machines relations"));
    // Update machines with manufacturer_id
    conn.execute(
        "UPDATE machines
        SET manufacturer_id = manufacturers.id
        FROM manufacturers
        JOIN extended_data ON extended_data.manufacturer = manufacturers.name
        WHERE extended_data.machine_name = machines.name",
        [],
    )?;
    // Update extended data with machine_id
    conn.execute(
        "UPDATE extended_data
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = extended_data.machine_name
         )",
        [],
    )?;
    // Update bios sets with machine_id
    conn.execute(
        "UPDATE bios_sets
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = bios_sets.machine_name
         )",
        [],
    )?;
    // Update roms with machine_id
    conn.execute(
        "UPDATE roms
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = roms.machine_name
         )",
        [],
    )?;
    // Update device refs with machine_id
    conn.execute(
        "UPDATE device_refs
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = device_refs.machine_name
         )",
        [],
    )?;
    // Update softwares with machine_id
    conn.execute(
        "UPDATE softwares
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = softwares.machine_name
         )",
        [],
    )?;
    // Update samples with machine_id
    conn.execute(
        "UPDATE samples
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = samples.machine_name
         )",
        [],
    )?;
    // Update disks with machine_id
    conn.execute(
        "UPDATE disks
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = disks.machine_name
         )",
        [],
    )?;
    // Update history sections with machine_id
    conn.execute(
        "UPDATE history_sections
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = history_sections.machine_name
         )",
        [],
    )?;
    // Update resources with machine_id
    conn.execute(
        "UPDATE resources
         SET machine_id = (
             SELECT id
             FROM machines
             WHERE machines.name = resources.machine_name
         )",
        [],
    )?;

    Ok(())
}
