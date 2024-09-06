use crate::models::Machine;
use std::{collections::HashMap, error::Error};

pub enum MachineFilter {
    Device,
    Bios,
    Mechanical,
    Modified,
    Clones,
    All,
}

pub fn remove_machines_by_filter(
    machines: &HashMap<String, Machine>,
    machine_filter: MachineFilter,
) -> Result<HashMap<String, Machine>, Box<dyn Error>> {
    if machines.is_empty() {
        return Err("No machines data loaded, please read the data first.".into());
    }

    let mut filtered_machines = HashMap::new();

    for (name, machine) in machines {
        if !filter_applies(machine, &machine_filter) {
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
        MachineFilter::All => {
            machine.is_device.unwrap_or(false)
                || machine.is_bios.unwrap_or(false)
                || machine.is_mechanical.unwrap_or(false)
                || is_modified_machine(&machine.description.as_ref().unwrap_or(&"".to_string()))
                || has_invalid_manufacturer(machine)
                || has_invalid_players(&machine)
                || is_clone(machine)
        }
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
