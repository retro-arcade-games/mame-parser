<h2 align="center">mame-parser</h2>

<p align="center">Simplify the management and processing of files containing MAME data</p>

<p align="center">
<img src="https://github.com/retro-arcade-games/mame-parser/actions/workflows/format-and-test.yml/badge.svg" />
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
- **Multi-format Exporting**: Export parsed data to various formats, including JSON, CSV, and SQLite.
- **Progress Tracking**: Monitor the progress of operations.

## Library Contents

### File Download

- **`download_file`**: Downloads a single MAME data file to a specified location.
- **`download_files`**: Downloads multiple MAME data files concurrently, providing progress tracking across multiple threads.

### File Unpacking

- **`unpack_file`**: Unpacks a single downloaded file from its archive format (e.g., ZIP or 7z) to a specified folder.
- **`unpack_files`**: Unpacks multiple files concurrently, allowing for efficient decompression with progress tracking.

## Getting Started

To get started with `mame-parser`, follow these steps:

### 1. Add the Dependency

Add `mame-parser` as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
mame-parser = "0.1"
```

Make sure to replace `"0.1"` with the actual version of `mame-parser` that you intend to use.

### 2. Import the Library

In your Rust project, import the necessary functions and types from the `mame-parser` crate:

```rust
use mame_parser::{
download_file, download_files, unpack_file, unpack_files,
ProgressInfo, CallbackType, ProgressCallback, SharedProgressCallback, MameDataType,
};
```

### 3. Example Usage

Here's a simple example demonstrating how to use `mame-parser` to download and unpack MAME data files:

```rust
use mame_parser::{download_file, unpack_file, MameDataType, ProgressCallback, ProgressInfo, CallbackType};
use std::path::Path;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
let workspace_path = Path::new("path/to/workspace");

    // Define a progress callback
    let progress_callback: ProgressCallback = Box::new(move |progress_info: ProgressInfo| {
        match progress_info.callback_type {
            CallbackType::Progress => println!("Progress: {} / {}", progress_info.progress, progress_info.total),
            CallbackType::Info => println!("Info: {}", progress_info.message),
            CallbackType::Finish => println!("Finished: {}", progress_info.message),
            CallbackType::Error => eprintln!("Error: {}", progress_info.message),
        }
    });

    // Download and unpack a MAME data file
    download_file(MameDataType::Series, &workspace_path, progress_callback.clone())?;
    unpack_file(MameDataType::Series, &workspace_path, progress_callback)?;

    Ok(())

}
```

This example shows how to:

- Define a progress callback function to monitor the progress of operations.
- Use `download_file` to download a MAME data file.
- Use `unpack_file` to unpack the downloaded file.

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
