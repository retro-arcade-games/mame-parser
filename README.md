<h2 align="center">mame-parser</h2>

<p align="center">Simplify the management and processing of files containing MAME data</p>

<p align="center">
    <a href="https://crates.io/crates/mame-parser">
        <img src="https://img.shields.io/crates/v/mame-parser.svg" />
    </a>
    <a href="https://docs.rs/mame-parser">
        <img src="https://docs.rs/mame-parser/badge.svg" />
    </a>
    <a href="https://github.com/retro-arcade-games/mame-parser/actions/workflows/format-and-test.yml">
        <img src="https://github.com/retro-arcade-games/mame-parser/actions/workflows/format-and-test.yml/badge.svg" />
    </a>
    <a href="https://github.com/retro-arcade-games/mame-parser/blob/main/LICENSE">
        <img src="https://img.shields.io/github/license/retro-arcade-games/mame-parser">
    </a>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#library-contents">Library Contents</a> •
  <a href="#getting-started">Getting Started</a> •
  <a href="#credits">Credits</a> •
  <a href="#contribute">Contribute</a> •
  <a href="#license">License</a>
</p>

`mame-parser` is a Rust library designed to simplify the management and processing of files containing MAME (Multiple Arcade Machine Emulator) data. This library provides a comprehensive suite of tools to automate the downloading, decompression, parsing, and exporting of MAME data, making it easier to handle and manipulate this information in various formats.

## Features

- **File Downloading**: Download the latest MAME-related files and store them in a specified location.
- **File Decompression**: Decompress downloaded files automatically, supporting multiple archive formats such as ZIP and 7z.
- **Data Parsing and Management**: Parse MAME data files with utilities for reading and processing information in-memory.
<!-- - **Multi-format Exporting**: Export parsed data to various formats, including JSON, CSV, and SQLite. -->
- **Progress Tracking**: Monitor the progress of operations.

## Library Contents

### File Handling

- **`download_file`**: Downloads a single MAME data file to a specified location.
- **`download_files`**: Downloads multiple MAME data files concurrently, providing progress tracking across multiple threads.
- **`unpack_file`**: Unpacks a single downloaded file from its archive format (e.g., ZIP or 7z) to a specified folder.
- **`unpack_files`**: Unpacks multiple files concurrently, allowing for efficient decompression with progress tracking.
- **`read_file`**: Reads a single data file and returns a Hashmap with the information.
- **`read_files`**: Reads multiple data files concurrently and returns a Hashmap with the information.

### Progress Tracking

Tools and types for tracking and managing progress updates during operations.

### MAME file readers

Functions for reading and parsing different MAME data file formats.

## Getting Started

To get started with `mame-parser`, follow these steps:

### 1. Add the Dependency

Add `mame-parser` as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
mame-parser = "0.5.0"
```

Make sure to replace `"0.5.0"` with the actual version of `mame-parser` that you intend to use.

### 2. Download file example

```rust
use mame_parser::file_handling::download_file;
use mame_parser::models::MameDataType;
use mame_parser::progress::{CallbackType, ProgressCallback, ProgressInfo};
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // Define the workspace directory
    let workspace_path = Path::new("playground");

    // Define a callback function for progress tracking
    let progress_callback: ProgressCallback = Box::new(move |progress_info: ProgressInfo| {
        // Update progress using console messages
        match progress_info.callback_type {
            CallbackType::Progress => {
                println!(
                    "Progress: {}/{}",
                    progress_info.progress, progress_info.total
                );
            }
            CallbackType::Info => {
                println!("Info: {}", progress_info.message);
            }
            CallbackType::Finish => {
                println!("Finished: {}", progress_info.message);
            }
            CallbackType::Error => {
                eprintln!("Error: {}", progress_info.message);
            }
        }
    });

    // Download the file
    let downloaded_file = download_file(MameDataType::Series, workspace_path, progress_callback);

    // Print the result
    match downloaded_file {
        Ok(downloaded_file) => {
            println!(
                "Downloaded file: {}",
                downloaded_file.as_path().to_str().unwrap()
            );
        }
        Err(e) => {
            eprintln!("Error during download: {}", e);
        }
    }

    Ok(())
}

```

### 4. Running the Example

To run the example, create a Rust project, add the code above to your `main.rs` file, and run:

```bash
cargo run
```

Make sure you have an active internet connection, as the example involves downloading files from the web.

## Credits

`mame-parser` wouldn't be possible without the invaluable contributions and resources provided by the following individuals and communities:

- **The MAME Community**: A special thanks to the entire MAME community for their continuous efforts in preserving arcade history and making it accessible to everyone. Your work is the foundation upon which this project is built.

- **AntoPISA and Progetto-SNAPS**: [AntoPISA](https://github.com/AntoPISA)'s [Progetto-SNAPS](https://www.progettosnaps.net) project has been an essential resource for MAME artwork and other assets. Thank you for your dedication and hard work in creating and maintaining this incredible resource.

- **Motoschifo and Arcade Database (ADB)**: Motoschifo's [Arcade Database](http://adb.arcadeitalia.net) is a comprehensive resource for MAME data, providing detailed information about arcade games and machines.

- **Arcade-History**: The team behind [Arcade-History](https://www.arcade-history.com) has done an amazing job in documenting the history of arcade games.

- **NPlayers Team**: The [NPlayers](https://nplayers.arcadebelgium.be) project by Arcade Belgium is a fantastic resource for information on multiplayer arcade games.

- **[zombiesbyte](https://github.com/zombiesbyte) and XMLTractor**: Special thanks to zombiesbyte for [XMLTractor](https://github.com/zombiesbyte/xmltractor) project.

## Contribute

Contributions are welcome! If you'd like to contribute, please fork the repository, create a new branch, and submit a pull request. Make sure to follow the project's coding guidelines and include tests where applicable.

### License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
