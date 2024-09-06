use crate::models::Machine;
use std::{collections::HashMap, error::Error};

/// Removes machines from the given HashMap based on a list of filter criteria.
///
/// This function takes a reference to a `HashMap` of machines and a slice of 
/// `MachineFilter` enums. It returns a new `HashMap` containing only the machines 
/// that do not match any of the specified filter criteria. If the input `machines` 
/// is empty, it returns an error.
///
/// # Arguments
///
/// * `machines` - A reference to a `HashMap` where the key is a `String` representing 
///   the machine's name, and the value is a `Machine` struct containing the machine details.
/// * `filters_to_remove` - A slice of `MachineFilter` enums that define the filter criteria 
///   for removing machines.
///
/// # Returns
///
/// * `Ok(HashMap<String, Machine>)` - A new `HashMap` containing the machines that 
///   do not match any of the filter criteria provided.
/// * `Err(Box<dyn Error>)` - An error if the input `machines` is empty.
///
/// # Errors
///
/// Returns an error if the input `machines` HashMap is empty. 
///
/// # Example
#[doc = docify::embed!("examples/remove_machines_by_filter.rs", main)]
///
pub fn remove_machines_by_filter(
    machines: &HashMap<String, Machine>,
    filters_to_remove: &[MachineFilter],
) -> Result<HashMap<String, Machine>, Box<dyn Error>> {
    if machines.is_empty() {
        return Err("No machines data loaded, please read the data first.".into());
    }

    let mut filtered_machines = HashMap::new();

    for (name, machine) in machines {
        let should_remove = filters_to_remove
            .iter()
            .any(|filter| filter_applies(machine, filter));

        if !should_remove {
            filtered_machines.insert(name.clone(), machine.clone());
        }
    }

    Ok(filtered_machines)
}

/// Removes machines from the given HashMap based on a list of categories to remove.
///
/// This function takes a reference to a `HashMap` of machines and a slice of 
/// `Category` enums. It returns a new `HashMap` containing only the machines 
/// that do not belong to any of the specified categories. If the input `machines` 
/// is empty, it returns an error.
///
/// # Arguments
///
/// * `machines` - A reference to a `HashMap` where the key is a `String` representing 
///   the machine's name, and the value is a `Machine` struct containing the machine details.
/// * `categories_to_remove` - A slice of `Category` enums that define the categories 
///   of machines to be removed.
///
/// # Returns
///
/// * `Ok(HashMap<String, Machine>)` - A new `HashMap` containing the machines that 
///   do not belong to any of the specified categories.
/// * `Err(Box<dyn Error>)` - An error if the input `machines` is empty.
///
/// # Errors
///
/// Returns an error if the input `machines` HashMap is empty. 
///
/// # Example
#[doc = docify::embed!("examples/remove_machines_by_category.rs", main)]
///
pub fn remove_machines_by_category(
    machines: &HashMap<String, Machine>,
    categories_to_remove: &[Category],
) -> Result<HashMap<String, Machine>, Box<dyn Error>> {
    if machines.is_empty() {
        return Err("No machines data loaded, please read the data first.".into());
    }

    let mut filtered_machines = HashMap::new();

    let categories_to_remove_str: Vec<&str> = categories_to_remove
        .iter()
        .map(|cat| cat.as_str())
        .collect();

    for (name, machine) in machines {
        let category_str = machine.category.as_deref();

        if category_str.is_some() && !categories_to_remove_str.contains(&category_str.unwrap()) {
            filtered_machines.insert(name.clone(), machine.clone());
        }
    }

    Ok(filtered_machines)
}

/// Checks if a given machine matches a specified filter criteria.
///
/// This function evaluates a `Machine` against a given `MachineFilter` and returns `true` 
/// if the machine matches the filter criteria, or `false` otherwise. Different filter 
/// criteria are evaluated based on the filter type, such as whether the machine is a 
/// device, BIOS, mechanical, modified, or a clone.
///
/// # Arguments
///
/// * `machine` - A reference to a `Machine` struct representing the machine to be evaluated.
/// * `machine_filter` - A reference to a `MachineFilter` enum specifying the filter criteria.
///
/// # Returns
///
/// * `bool` - `true` if the machine matches the filter criteria; `false` otherwise.
///
fn filter_applies(machine: &Machine, machine_filter: &MachineFilter) -> bool {
    match machine_filter {
        MachineFilter::Device => machine.is_device.unwrap_or(false),
        MachineFilter::Bios => machine.is_bios.unwrap_or(false),
        MachineFilter::Mechanical => machine.is_mechanical.unwrap_or(false),
        MachineFilter::Modified => {
            is_modified_machine(&machine.description.as_ref().unwrap_or(&"".to_string()))
                || has_invalid_manufacturer(machine)
                || has_invalid_players(&machine)
        }
        MachineFilter::Clones => is_clone(machine),
    }
}

/// Determines if a machine is considered "modified" based on its description.
///
/// This function checks if the provided machine description contains any keywords 
/// that indicate the machine is a modified version, such as "bootleg," "PlayChoice-10," 
/// "Nintendo Super System," or "prototype". The check is case-insensitive.
///
/// # Arguments
///
/// * `description` - A reference to a `str` representing the description of the machine.
///
/// # Returns
///
/// * `bool` - `true` if the description contains any keyword that indicates the machine is modified; 
///   `false` otherwise.
///
/// # Keywords Checked
///
/// The function checks for the following keywords to determine if a machine is modified:
/// * "bootleg"
/// * "PlayChoice-10"
/// * "Nintendo Super System"
/// * "prototype"
///
/// These keywords are matched in a case-insensitive manner.
fn is_modified_machine(description: &str) -> bool {
    let modified_keywords = vec![
        "bootleg",
        "PlayChoice-10",
        "Nintendo Super System",
        "prototype",
    ];
    for keyword in modified_keywords {
        if description.to_lowercase().contains(&keyword.to_lowercase()) {
            return true;
        }
    }
    false
}

/// Checks if a machine has an invalid manufacturer.
///
/// This function examines the manufacturer of a given `Machine` to determine if it 
/// matches any of the known invalid manufacturers, such as "unknown" or "bootleg". 
/// The check is performed in a case-insensitive manner.
///
/// # Arguments
///
/// * `machine` - A reference to a `Machine` struct whose manufacturer is to be evaluated.
///
/// # Returns
///
/// * `bool` - `true` if the machine's manufacturer is considered invalid; 
///   `false` otherwise.
///
/// # Invalid Manufacturers
///
/// The function checks for the following invalid manufacturers:
/// * "unknown"
/// * "bootleg"
///
/// These keywords are matched in a case-insensitive manner.
fn has_invalid_manufacturer(machine: &Machine) -> bool {
    let invalid_manufacturers = vec!["unknown", "bootleg"];
    // Check if manufacturer in machine is invalid
    if let Some(manufacturer) = &machine.manufacturer {
        for invalid_manufacturer in invalid_manufacturers {
            if manufacturer
                .to_lowercase()
                .contains(&invalid_manufacturer.to_lowercase())
            {
                return true;
            }
        }
    }
    false
}

/// Checks if a machine has an invalid player type.
///
/// This function evaluates the `players` field of a given `Machine` to determine if 
/// it matches any of the known invalid player types, such as "BIOS", "Device", or "Non-arcade". 
/// The check is performed in a case-insensitive manner.
///
/// # Arguments
///
/// * `machine` - A reference to a `Machine` struct whose `players` field is to be evaluated.
///
/// # Returns
///
/// * `bool` - `true` if the machine's player type is considered invalid; 
///   `false` otherwise.
///
/// # Invalid Player Types
///
/// The function checks for the following invalid player types:
/// * "BIOS"
/// * "Device"
/// * "Non-arcade"
///
/// These types are matched in a case-insensitive manner.
fn has_invalid_players(machine: &Machine) -> bool {
    let invalid_players = vec!["BIOS", "Device", "Non-arcade"];
    if let Some(players) = &machine.players {
        for invalid_player in invalid_players {
            if players
                .to_lowercase()
                .contains(&invalid_player.to_lowercase())
            {
                return true;
            }
        }
    }
    false
}

/// Determines if a machine is a clone of another machine.
///
/// This function checks if a given `Machine` is a clone by evaluating whether the 
/// `clone_of` or `rom_of` fields are set to `Some`, indicating that the machine is 
/// derived from or is a variant of another machine.
///
/// # Arguments
///
/// * `machine` - A reference to a `Machine` struct to be evaluated.
///
/// # Returns
///
/// * `bool` - `true` if the machine is considered a clone; `false` otherwise.
///
/// # Clone Determination
///
/// The function determines that a machine is a clone if either of the following conditions is true:
/// * The `clone_of` field is `Some`, indicating it has a parent machine.
/// * The `rom_of` field is `Some`, indicating it shares ROM data with another machine.
fn is_clone(machine: &Machine) -> bool {
    machine.clone_of.is_some() || machine.rom_of.is_some()
}

/// Represents different filter criteria for filtering machines.
///
/// The `MachineFilter` enum defines various criteria that can be used to filter 
/// machines based on specific attributes, such as whether the machine is a device, 
/// BIOS, mechanical, modified, or a clone.
///
/// # Variants
///
/// * `Device` - Filters machines that are marked as devices.
/// * `Bios` - Filters machines that are identified as BIOS.
/// * `Mechanical` - Filters machines that are categorized as mechanical.
/// * `Modified` - Filters machines that are considered modified based on their description, 
///   manufacturer validity, or player information.
/// * `Clones` - Filters machines that are identified as clones of other machines.
///
pub enum MachineFilter {
    /// Filters machines that are marked as devices.
    Device,
    /// Filters machines that are identified as BIOS.
    Bios,
    /// Filters machines that are categorized as mechanical.
    Mechanical,
    /// Filters machines that are considered modified based on their description,
    Modified,
    /// Filters machines that are identified as clones of other machines.
    Clones,
}

/// Represents the different categories a machine can belong to.
///
/// The `Category` enum defines various categories that a machine can be classified into. 
/// These categories represent different types of machines or devices, such as arcade games, 
/// simulators, computers, gambling machines, and more.
///
/// # Variants
///
/// * `Arcade` - Represents arcade machines.
/// * `BallAndPaddle` - Represents machines involving ball and paddle gameplay.
/// * `BoardGame` - Represents board game machines.
/// * `Calculator` - Represents calculator devices.
/// * `CardGames` - Represents machines for playing card games.
/// * `Climbing` - Represents climbing-related machines.
/// * `Computer` - Represents general-purpose computer machines.
/// * `ComputerGraphicWorkstation` - Represents computer graphic workstations.
/// * `DigitalCamera` - Represents digital cameras.
/// * `DigitalSimulator` - Represents digital simulators.
/// * `Driving` - Represents driving simulation machines.
/// * `Electromechanical` - Represents electromechanical machines.
/// * `Fighter` - Represents fighting game machines.
/// * `Gambling` - Represents gambling machines.
/// * `Game` - Represents general game machines.
/// * `GameConsole` - Represents game consoles.
/// * `GameConsoleComputer` - Represents devices that function as both a game console and a computer.
/// * `Handheld` - Represents handheld gaming devices.
/// * `Maze` - Represents maze-type game machines.
/// * `MedalGame` - Represents medal-based game machines.
/// * `MedicalEquipment` - Represents machines used as medical equipment.
/// * `Misc` - Represents miscellaneous machines not covered by other categories.
/// * `MultiGame` - Represents machines that support multiple games.
/// * `Multiplay` - Represents multiplayer gaming machines.
/// * `Music` - Represents music-related devices.
/// * `MusicGame` - Represents machines specifically for music games.
/// * `Platform` - Represents platform game machines.
/// * `Player` - Represents player-specific machines.
/// * `Printer` - Represents printing devices.
/// * `Puzzle` - Represents puzzle game machines.
/// * `Quiz` - Represents quiz game machines.
/// * `Radio` - Represents radio devices.
/// * `RedemptionGame` - Represents redemption game machines.
/// * `Shooter` - Represents shooting game machines.
/// * `Simulation` - Represents simulation machines.
/// * `SlotMachine` - Represents slot machines.
/// * `Sports` - Represents sports game machines.
/// * `System` - Represents system-level devices or machines.
/// * `TTLBallAndPaddle` - Represents TTL (Transistor-Transistor Logic) ball and paddle games.
/// * `TTLDriving` - Represents TTL driving games.
/// * `TTLMaze` - Represents TTL maze games.
/// * `TTLQuiz` - Represents TTL quiz games.
/// * `TTLShooter` - Represents TTL shooter games.
/// * `TTLSports` - Represents TTL sports games.
/// * `TVBundle` - Represents machines or devices bundled with a TV.
/// * `Tablet` - Represents tablet devices.
/// * `Tabletop` - Represents tabletop game machines.
/// * `Telephone` - Represents telephone devices.
/// * `Touchscreen` - Represents touchscreen devices.
/// * `Utilities` - Represents utility devices.
/// * `Watch` - Represents watches or watch-like devices.
/// * `WhacAMole` - Represents Whac-A-Mole machines.
#[derive(Debug)]
pub enum Category {
    Arcade,
    BallAndPaddle,
    BoardGame,
    Calculator,
    CardGames,
    Climbing,
    Computer,
    ComputerGraphicWorkstation,
    DigitalCamera,
    DigitalSimulator,
    Driving,
    Electromechanical,
    Fighter,
    Gambling,
    Game,
    GameConsole,
    GameConsoleComputer,
    Handheld,
    Maze,
    MedalGame,
    MedicalEquipment,
    Misc,
    MultiGame,
    Multiplay,
    Music,
    MusicGame,
    Platform,
    Player,
    Printer,
    Puzzle,
    Quiz,
    Radio,
    RedemptionGame,
    Shooter,
    Simulation,
    SlotMachine,
    Sports,
    System,
    TTLBallAndPaddle,
    TTLDriving,
    TTLMaze,
    TTLQuiz,
    TTLShooter,
    TTLSports,
    TVBundle,
    Tablet,
    Tabletop,
    Telephone,
    Touchscreen,
    Utilities,
    Watch,
    WhacAMole,
}

impl Category {
    /// Returns the string representation of the `Category` enum variant.
    fn as_str(&self) -> &'static str {
        match self {
            Category::Arcade => "Arcade",
            Category::BallAndPaddle => "Ball & Paddle",
            Category::BoardGame => "Board Game",
            Category::Calculator => "Calculator",
            Category::CardGames => "Card Games",
            Category::Climbing => "Climbing",
            Category::Computer => "Computer",
            Category::ComputerGraphicWorkstation => "Computer Graphic Workstation",
            Category::DigitalCamera => "Digital Camera",
            Category::DigitalSimulator => "Digital Simulator",
            Category::Driving => "Driving",
            Category::Electromechanical => "Electromechanical",
            Category::Fighter => "Fighter",
            Category::Gambling => "Gambling",
            Category::Game => "Game",
            Category::GameConsole => "Game Console",
            Category::GameConsoleComputer => "Game Console/Computer",
            Category::Handheld => "Handheld",
            Category::Maze => "Maze",
            Category::MedalGame => "Medal Game",
            Category::MedicalEquipment => "Medical Equipment",
            Category::Misc => "Misc.",
            Category::MultiGame => "MultiGame",
            Category::Multiplay => "Multiplay",
            Category::Music => "Music",
            Category::MusicGame => "Music Game",
            Category::Platform => "Platform",
            Category::Player => "Player",
            Category::Printer => "Printer",
            Category::Puzzle => "Puzzle",
            Category::Quiz => "Quiz",
            Category::Radio => "Radio",
            Category::RedemptionGame => "Redemption Game",
            Category::Shooter => "Shooter",
            Category::Simulation => "Simulation",
            Category::SlotMachine => "Slot Machine",
            Category::Sports => "Sports",
            Category::System => "System",
            Category::TTLBallAndPaddle => "TTL * Ball & Paddle",
            Category::TTLDriving => "TTL * Driving",
            Category::TTLMaze => "TTL * Maze",
            Category::TTLQuiz => "TTL * Quiz",
            Category::TTLShooter => "TTL * Shooter",
            Category::TTLSports => "TTL * Sports",
            Category::TVBundle => "TV Bundle",
            Category::Tablet => "Tablet",
            Category::Tabletop => "Tabletop",
            Category::Telephone => "Telephone",
            Category::Touchscreen => "Touchscreen",
            Category::Utilities => "Utilities",
            Category::Watch => "Watch",
            Category::WhacAMole => "Whac-A-Mole",
        }
    }
}
