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

/// Writes machine data to a SQLite database.
///
/// This function exports the contents of a `HashMap` of `Machine` data to a SQLite database file.
/// The function creates a new SQLite database at the specified path, inserts all machine data,
/// and then establishes necessary relationships. Progress updates are provided through a callback function.
///
/// # Parameters
/// - `data_base_path`: A `&str` representing the file path where the SQLite database will be created.
/// - `machines`: A reference to a `HashMap<String, Machine>` containing all machine data to be exported.
///   The key is the machine name, and the value is a `Machine` struct with all associated metadata.
/// - `progress_callback`: A callback function of type `ProgressCallback` that provides progress updates during the SQLite writing process.
///   The callback receives a `ProgressInfo` struct containing fields like `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<(), Box<dyn Error + Send + Sync>>`:
/// - On success: Returns `Ok(())` after successfully writing all data to the SQLite database.
/// - On failure: Returns an error if there are issues creating the database, writing data, or establishing relationships.
///
/// # Errors
/// This function will return an error if:
/// - The `machines` HashMap is empty, indicating that there is no data to write.
/// - There are any I/O errors when creating the SQLite database file.
/// - The database connection or transactions fail during the writing process.
/// - The progress callback fails to execute correctly during any phase of the writing process.
///
/// # SQLite Database Structure
/// The SQLite database includes:
/// - Tables for machine data, each containing relevant metadata like name, source file, manufacturer, etc.
/// - Relationships between machines and additional attributes such as languages and players.
/// - Data is inserted in batches to optimize performance and reduce memory usage.
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

/// Creates the necessary tables in the SQLite database.
///
/// This function initializes the SQLite database by creating all the required tables for storing machine data,
/// including tables for series, categories, subcategories, manufacturers, languages, players, and other related data.
/// The tables are created only if they do not already exist, ensuring that existing data is not overwritten.
///
/// # Parameters
/// - `conn`: A mutable reference to a `Connection` representing the SQLite database connection.
///
/// # Returns
/// Returns a `Result<()>`:
/// - On success: Returns `Ok(())` after successfully creating all the necessary tables.
/// - On failure: Returns an error if there is an issue executing any of the SQL statements to create the tables.
///
/// # Errors
/// This function will return an error if:
/// - There is an I/O issue with the database file.
/// - The SQL execution fails due to syntax errors or constraint violations.
///
/// # Tables Created
/// - `series`: Stores series information with unique names.
/// - `categories`: Stores category names with unique constraints.
/// - `subcategories`: Stores subcategory names associated with categories.
/// - `manufacturers`: Stores manufacturer names with unique constraints.
/// - `languages`: Stores language names with unique constraints.
/// - `players`: Stores player information with unique names.
/// - `machines`: Stores main machine data, including references to series, categories, subcategories, and manufacturers.
/// - `machine_languages`: Stores relationships between machines and languages.
/// - `machine_players`: Stores relationships between machines and players.
/// - `extended_data`: Stores additional normalized data for machines.
/// - `bios_sets`: Stores BIOS set information linked to each machine.
/// - `roms`: Stores ROM-specific data for each machine.
/// - `device_refs`: Stores device reference data for each machine.
/// - `softwares`: Stores software information linked to each machine.
/// - `samples`: Stores sample data for each machine.
/// - `disks`: Stores disk information for each machine.
/// - `history_sections`: Stores historical sections related to each machine.
/// - `resources`: Stores resource information such as size, type, and checksums for each machine.
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

/// Inserts machine data into the SQLite database.
///
/// This function inserts all relevant data for a given `Machine` into the corresponding tables in the SQLite database.
/// The function handles the insertion of main machine data, as well as related data such as extended data, BIOS sets, ROMs, device references, software, samples, disks, history sections, and resources.
/// Existing entries are replaced if there are conflicts to ensure the data is up-to-date.
///
/// # Parameters
/// - `transaction`: A reference to a `Transaction` object representing an active SQLite transaction.
///   This transaction is used to perform multiple insertions atomically.
/// - `machine`: A reference to a `Machine` struct containing all the data to be inserted into the database.
///
/// # Returns
/// Returns a `Result<()>`:
/// - On success: Returns `Ok(())` after successfully inserting all machine data.
/// - On failure: Returns an error if there are issues executing any of the SQL statements.
///
/// # Errors
/// This function will return an error if:
/// - There are SQL syntax errors or constraint violations during the insert operations.
/// - The transaction encounters any I/O issues or other database errors.
///
/// # Inserted Data
/// - `machines`: Inserts or replaces the main machine data.
/// - `extended_data`: Inserts or replaces extended data associated with the machine.
/// - `bios_sets`: Inserts or replaces BIOS set information linked to the machine.
/// - `roms`: Inserts or replaces ROM-specific data for the machine.
/// - `device_refs`: Inserts or replaces device references for the machine.
/// - `softwares`: Inserts or replaces software information linked to the machine.
/// - `samples`: Inserts or replaces sample data for the machine.
/// - `disks`: Inserts or replaces disk information for the machine.
/// - `history_sections`: Inserts or replaces historical sections related to the machine.
/// - `resources`: Inserts or replaces resource information such as size, type, and checksums for the machine.
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

/// Extracts languages from the machine data and inserts them into the SQLite database.
///
/// This function processes all the machines in the provided `HashMap` to extract a unique list of languages.
/// It then inserts each language into the `languages` table in the SQLite database.
/// If a language already exists in the table, the insertion is ignored to avoid duplication.
///
/// # Parameters
/// - `conn`: A mutable reference to a `Connection` representing the SQLite database connection.
/// - `machines`: A reference to a `HashMap<String, Machine>` containing all the machine data, from which the languages will be extracted.
///
/// # Returns
/// Returns a `Result<()>`:
/// - On success: Returns `Ok(())` after successfully inserting all unique languages into the database.
/// - On failure: Returns an error if there are issues executing any of the SQL statements.
///
/// # Errors
/// This function will return an error if:
/// - There are SQL syntax errors or constraint violations during the insert operations.
/// - The transaction encounters any I/O issues or other database errors.
///
/// # Inserted Data
/// - `languages`: Inserts each unique language extracted from the machine data into the `languages` table.
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

/// Inserts relationships between machines and languages into the SQLite database.
///
/// This function establishes relationships between machines and their associated languages in the `machine_languages` table.
/// It queries the `machines` table to retrieve the machine IDs and their associated languages,
/// then inserts a record for each machine-language pair. The insertion links each machine to the corresponding language ID from the `languages` table.
///
/// # Parameters
/// - `conn`: A mutable reference to a `Connection` representing the SQLite database connection.
///
/// # Returns
/// Returns a `Result<()>`:
/// - On success: Returns `Ok(())` after successfully inserting all machine-language relationships into the database.
/// - On failure: Returns an error if there are issues executing any of the SQL statements.
///
/// # Errors
/// This function will return an error if:
/// - There are SQL syntax errors or constraint violations during the insert operations.
/// - The transaction encounters any I/O issues or other database errors.
/// - The SQL query fails to retrieve machine IDs or language names correctly.
///
/// # Inserted Data
/// - `machine_languages`: Inserts records associating each machine with its respective languages in the `machine_languages` table.
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

/// Extracts player information from the machine data and inserts it into the SQLite database.
///
/// This function processes all the machines in the provided `HashMap` to extract a unique list of player types.
/// It then inserts each player type into the `players` table in the SQLite database.
/// If a player type already exists in the table, the insertion is ignored to avoid duplication.
///
/// # Parameters
/// - `conn`: A mutable reference to a `Connection` representing the SQLite database connection.
/// - `machines`: A reference to a `HashMap<String, Machine>` containing all the machine data, from which the player types will be extracted.
///
/// # Returns
/// Returns a `Result<()>`:
/// - On success: Returns `Ok(())` after successfully inserting all unique player types into the database.
/// - On failure: Returns an error if there are issues executing any of the SQL statements.
///
/// # Errors
/// This function will return an error if:
/// - There are SQL syntax errors or constraint violations during the insert operations.
/// - The transaction encounters any I/O issues or other database errors.
///
/// # Inserted Data
/// - `players`: Inserts each unique player type extracted from the machine data into the `players` table.
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

/// Inserts relationships between machines and players into the SQLite database.
///
/// This function establishes relationships between machines and their associated player types in the `machine_players` table.
/// It queries the `machines` and `extended_data` tables to retrieve the machine IDs and their associated player types,
/// then inserts a record for each machine-player pair. The insertion links each machine to the corresponding player ID from the `players` table.
///
/// # Parameters
/// - `conn`: A mutable reference to a `Connection` representing the SQLite database connection.
///
/// # Returns
/// Returns a `Result<()>`:
/// - On success: Returns `Ok(())` after successfully inserting all machine-player relationships into the database.
/// - On failure: Returns an error if there are issues executing any of the SQL statements.
///
/// # Errors
/// This function will return an error if:
/// - There are SQL syntax errors or constraint violations during the insert operations.
/// - The transaction encounters any I/O issues or other database errors.
/// - The SQL query fails to retrieve machine IDs or player names correctly.
///
/// # Inserted Data
/// - `machine_players`: Inserts records associating each machine with its respective player types in the `machine_players` table.
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

/// Creates and updates relationships between different entities in the SQLite database.
///
/// This function establishes and updates various relationships between entities such as machines, categories, subcategories, series, and manufacturers.
/// It performs multiple SQL operations to insert unique entries into the respective tables and updates foreign key references in the `machines` table and other related tables.
/// Progress updates are provided through a callback function to indicate the progress of each operation.
///
/// # Parameters
/// - `conn`: A mutable reference to a `Connection` representing the SQLite database connection.
/// - `machines`: A reference to a `HashMap<String, Machine>` containing all machine data for creating and updating relationships.
/// - `progress_callback`: A reference to a callback function of type `ProgressCallback` that provides progress updates during the process of creating and updating relations.
///
/// # Returns
/// Returns a `Result<()>`:
/// - On success: Returns `Ok(())` after successfully creating and updating all relationships.
/// - On failure: Returns an error if there are issues executing any of the SQL statements.
///
/// # Errors
/// This function will return an error if:
/// - There are SQL syntax errors or constraint violations during the insert or update operations.
/// - The transaction encounters any I/O issues or other database errors.
/// - The progress callback fails to execute correctly during any phase of the relationship creation process.
///
/// # Created and Updated Data
/// - `categories`: Inserts unique categories from the `machines` table and updates machines with the corresponding `category_id`.
/// - `subcategories`: Inserts unique subcategories associated with categories and updates machines with the corresponding `subcategory_id`.
/// - `series`: Inserts unique series names and updates machines with the corresponding `series_id`.
/// - `manufacturers`: Inserts unique manufacturer names from the `extended_data` and updates machines with the corresponding `manufacturer_id`.
/// - Updates various tables (`bios_sets`, `roms`, `device_refs`, `softwares`, `samples`, `disks`, `history_sections`, `resources`) to link their records with the correct `machine_id`.
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
