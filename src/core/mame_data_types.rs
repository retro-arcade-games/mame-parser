use regex::Regex;

pub enum MameDataType {
    Mame,
    Languages,
    NPlayers,
    Catver,
    Series,
    History,
    Resources,
}

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

pub struct MameDataTypeDetails {
    pub name: &'static str,
    pub source: &'static str,
    pub source_match: &'static str,
    pub zip_file_pattern: Regex,
    pub data_file_pattern: Regex,
    // pub read_function: fn(&str) -> Result<(), Box<dyn std::error::Error>>,
}

/// Get the details of a MameDataType
/// 
/// # Parameters
/// * `data_type` - The MameDataType to get the details for
/// 
/// # Returns
/// The MameDataTypeDetails for the given MameDataType
/// 
/// # Examples
/// ```
/// use mame_parser::core::mame_data_types::{MameDataType, get_data_type_details};
/// 
/// let mame_details = get_data_type_details(MameDataType::Mame);
/// assert_eq!(mame_details.name, "Mame");
/// ```
///
pub fn get_data_type_details(data_type: MameDataType) -> MameDataTypeDetails {
    match data_type {
        MameDataType::Mame => MameDataTypeDetails {
            name: "Mame",
            source: "https://www.progettosnaps.net/dats/MAME",
            source_match: "download/?tipo=dat_mame&file=/dats/MAME/packs/MAME_Dats",
            zip_file_pattern: Regex::new(r"^MAME_Dats_\d+\.7z$").unwrap(),
            data_file_pattern: Regex::new(r"MAME\s+[0-9]*\.[0-9]+\.dat").unwrap(),
            // read_function: mame_reader::read_mame_file,
        },
        MameDataType::Languages => MameDataTypeDetails {
            name: "Languages",
            source: "https://www.progettosnaps.net/languages",
            source_match: "download",
            zip_file_pattern: Regex::new(r"^pS_Languages_\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"languages.ini").unwrap(),
            // read_function: languages_reader::read_languages_file,
        },
        MameDataType::NPlayers => MameDataTypeDetails {
            name: "NPlayers",
            source: "http://nplayers.arcadebelgium.be",
            source_match: "files",
            zip_file_pattern: Regex::new(r"^nplayers0\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"nplayers.ini").unwrap(),
            // read_function: nplayers_reader::read_nplayers_file,
        },
        MameDataType::Catver => MameDataTypeDetails {
            name: "Catver",
            source: "https://www.progettosnaps.net/catver",
            source_match: "download",
            zip_file_pattern: Regex::new(r"^pS_CatVer_\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"catver.ini").unwrap(),
            // read_function: catver_reader::read_catver_file,
        },
        MameDataType::Series => MameDataTypeDetails {
            name: "Series",
            source: "https://www.progettosnaps.net/series",
            source_match: "download",
            zip_file_pattern: Regex::new(r"^pS_Series_\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"series.ini").unwrap(),
            // read_function: series_reader::read_series_file,
        },
        MameDataType::History => MameDataTypeDetails {
            name: "History",
            source: "https://www.arcade-history.com/index.php?page=download",
            source_match: "dats",
            zip_file_pattern: Regex::new(r"^history\d+\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"history.xml").unwrap(),
            // read_function: history_reader::read_history_file,
        },
        MameDataType::Resources => MameDataTypeDetails {
            name: "Resources",
            source: "https://www.progettosnaps.net/dats",
            source_match: "download/?tipo=dat_resource&file=/dats/cmdats/pS_AllProject_",
            zip_file_pattern: Regex::new(r"^pS_AllProject_\d{8}_\d+_\([a-zA-Z]+\)\.zip$").unwrap(),
            data_file_pattern: Regex::new(r"^pS_AllProject_\d{8}_\d+_\([a-zA-Z]+\)\.dat$").unwrap(),
            // read_function: resources_reader::read_resources_file,
        }
    }
}
