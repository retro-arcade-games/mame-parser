use crate::models::Machine;
use std::{collections::HashMap, error::Error};

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

fn is_clone(machine: &Machine) -> bool {
    machine.clone_of.is_some() || machine.rom_of.is_some()
}

/// Represents the different filters that can be applied to arcade machines.
pub enum MachineFilter {
    Device,
    Bios,
    Mechanical,
    Modified,
    Clones,
}

/// Represents the different categories of arcade machines.
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
    /// Returns the string representation of the category.
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
