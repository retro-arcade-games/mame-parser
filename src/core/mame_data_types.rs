use crate::core::callback_progress::ProgressCallback;
use crate::core::models::Machine;
use crate::core::readers::{
    catver_reader, history_reader, languages_reader, mame_reader, nplayers_reader,
    resources_reader, series_reader,
};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;

/// Represents different types of MAME data that can be downloaded and processed.
///
/// The `MameDataType` enum categorizes the various data types associated with MAME, each of which
/// may have specific sources, file formats, and processing requirements. This enum is used throughout
/// the application to identify and work with these distinct data categories.
///
/// # Variants
/// - `Mame`: Represents the core MAME data, typically including ROM information and basic metadata.
/// - `Languages`: Represents data related to language support in MAME, such as localization files.
/// - `NPlayers`: Represents data about the number of players supported by each game (e.g., single-player, multiplayer).
/// - `Catver`: Represents category and version data, often used for classifying and organizing games.
/// - `Series`: Represents data related to game series, grouping related titles together.
/// - `History`: Represents historical data, trivia, and other contextual information related to games.
/// - `Resources`: Represents additional resources like images, videos, and other media related to MAME games.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MameDataType {
    /// Represents the core MAME data, including ROM information and basic metadata.
    Mame,
    /// Represents language-specific data, such as localization files for MAME.
    Languages,
    /// Represents data regarding the number of players supported by each game (e.g., single-player, multiplayer).
    NPlayers,
    /// Represents category and version data used for organizing and classifying MAME games.
    Catver,
    /// Represents data related to game series, grouping related titles together.
    Series,
    /// Represents historical data, trivia, and contextual information for MAME games.
    History,
    /// Represents additional resources like images, videos, and other media related to MAME games.
    Resources,
}

/// Returns a slice containing all variants of the `MameDataType` enum.
///
/// This function provides a static reference to an array containing every possible variant of the `MameDataType` enum.
/// It is useful when you need to iterate over or perform operations on all data types managed by the application.
///
/// # Returns
/// A static slice (`&'static [MameDataType]`) containing all `MameDataType` variants.
///
impl MameDataType {
    pub fn all_variants() -> &'static [MameDataType] {
        &[
            MameDataType::Mame,
            MameDataType::Languages,
            MameDataType::NPlayers,
            MameDataType::Catver,
            MameDataType::Series,
            MameDataType::History,
            MameDataType::Resources,
        ]
    }
}

/// Represents the details associated with a specific MAME data type.
///
/// The `MameDataTypeDetails` struct holds information necessary to manage the downloading and processing
/// of MAME data files. It includes metadata such as the name of the data type, the URL source, patterns
/// for matching specific files, and an optional function pointer for reading and processing the downloaded data.
///
/// # Fields
/// - `name`: A static string slice (`&'static str`) representing the name of the MAME data type (e.g., "ROMs", "DAT Files").
/// - `source`: A static string slice (`&'static str`) representing the URL source from which the data can be downloaded.
/// - `source_match`: A static string slice (`&'static str`) used as a substring to match the relevant download link.
/// - `zip_file_pattern`: A `Regex` pattern that matches the specific zip files associated with this data type.
/// - `data_file_pattern`: A `Regex` pattern that matches the internal files within the downloaded zip files.
/// - `read_function`: A function pointer of type `fn(&str) -> Result<(), Box<dyn std::error::Error>>`
///   that is intended to read and process the extracted data file. This can be used to invoke specific parsers or handlers
///   based on the data type.
///
pub struct MameDataTypeDetails {
    pub name: &'static str,
    pub source: &'static str,
    pub source_match: &'static str,
    pub zip_file_pattern: Regex,
    pub data_file_pattern: Regex,
    pub read_function: fn(
        file_path: &str,
        progress_callback: ProgressCallback,
    ) -> Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>,
}

/// Retrieves the details for a given `MameDataType`.
///
/// This function returns the relevant metadata for a specific `MameDataType`,
/// such as the name, source URL, patterns to match the expected ZIP files,
/// and the data files inside the archive. These details are used to locate
/// and process the appropriate files for each type.
///
/// # Parameters
/// - `data_type`: The `MameDataType` for which the details are being retrieved.
///
/// # Returns
/// A `MameDataTypeDetails` struct containing the following information:
/// - `name`: The name of the data type (e.g., "Mame", "Languages").
/// - `source`: The URL from which the file is downloaded.
/// - `source_match`: A pattern or additional path used to determine the exact file to download.
/// - `zip_file_pattern`: A regex pattern that matches the ZIP file name.
/// - `data_file_pattern`: A regex pattern that matches the data file inside the ZIP archive.
///
pub(crate) fn get_data_type_details(data_type: MameDataType) -> MameDataTypeDetails {
    match data_type {
        MameDataType::Mame => MameDataTypeDetails {
            name: "Mame",
            source: "https://www.progettosnaps.net/dats/MAME",
            source_match: "download/?tipo=dat_mame&file=/dats/MAME/packs/MAME_Dats",
            zip_file_pattern: Regex::new(r"^MAME_Dats_\d+\.7z$").unwrap(),
            data_file_pattern: Regex::new(r"MAME\s+[0-9]*\.[0-9]+\.dat").unwrap(),
            read_function: mame_reader::read_mame_file,
        },
        MameDataType::Languages => MameDataTypeDetails {
            name: "Languages",
            source: "https://www.progettosnaps.net/languages",
            source_match: "download",
            zip_file_pattern: Regex::new(r"^pS_Languages_\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"languages.ini").unwrap(),
            read_function: languages_reader::read_languages_file,
        },
        MameDataType::NPlayers => MameDataTypeDetails {
            name: "NPlayers",
            source: "http://nplayers.arcadebelgium.be",
            source_match: "files",
            zip_file_pattern: Regex::new(r"^nplayers0\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"nplayers.ini").unwrap(),
            read_function: nplayers_reader::read_nplayers_file,
        },
        MameDataType::Catver => MameDataTypeDetails {
            name: "Catver",
            source: "https://www.progettosnaps.net/catver",
            source_match: "download",
            zip_file_pattern: Regex::new(r"^pS_CatVer_\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"catver.ini").unwrap(),
            read_function: catver_reader::read_catver_file,
        },
        MameDataType::Series => MameDataTypeDetails {
            name: "Series",
            source: "https://www.progettosnaps.net/series",
            source_match: "download",
            zip_file_pattern: Regex::new(r"^pS_Series_\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"series.ini").unwrap(),
            read_function: series_reader::read_series_file,
        },
        MameDataType::History => MameDataTypeDetails {
            name: "History",
            source: "https://www.arcade-history.com/index.php?page=download",
            source_match: "dats",
            zip_file_pattern: Regex::new(r"^history\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"history.xml").unwrap(),
            read_function: history_reader::read_history_file,
        },
        MameDataType::Resources => MameDataTypeDetails {
            name: "Resources",
            source: "https://www.progettosnaps.net/dats",
            source_match: "download/?tipo=dat_resource&file=/dats/cmdats/pS_AllProject_",
            zip_file_pattern: Regex::new(r"^pS_AllProject_\d{8}_\d+_\([a-zA-Z]+\)\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"^pS_AllProject_\d{8}_\d+_\([a-zA-Z]+\)\.dat$").unwrap(),
            read_function: resources_reader::read_resources_file,
        },
    }
}
