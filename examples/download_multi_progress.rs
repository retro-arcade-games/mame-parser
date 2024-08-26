use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mame_parser::core::downloader::file_downloader::download_file_callback;
use mame_parser::core::mame_data_types::MameDataType;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::thread;

fn spawn_download(
    data_type: MameDataType,
    destination_folder: PathBuf,
    progress_bar: ProgressBar,
) -> thread::JoinHandle<Result<PathBuf, Box<dyn Error + Send + Sync>>> {
    thread::spawn(move || {
        download_file_callback(
            data_type,
            &destination_folder,
            Some(move |downloaded, total_size, message:String| {

                if message.len() > 0 {
                    progress_bar.set_message(message);
                }else{
                    progress_bar.set_message("");
                    if total_size > 0 {
                        progress_bar.set_length(total_size);
                    }
                    progress_bar.set_position(downloaded);
    
                    if downloaded == total_size {
                        progress_bar.finish_with_message(format!("{:?} Download completed", data_type));
                    }
                }

            }),
        )
    })
}

fn create_progress_bar(
    multi_progress: &MultiProgress,
    label: &str,
) -> ProgressBar {
    let progress_bar = multi_progress.add(ProgressBar::new(100));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.green}} {} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}}) {{msg}}",
                label
            ))
            .progress_chars("#>-"),
    );
    progress_bar
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let destination_folder = Path::new("playground/downloads");
    std::fs::create_dir_all(destination_folder)?;

    let multi_progress = MultiProgress::new();

    let resources_progress_bar = create_progress_bar(&multi_progress, "Resources");
    let nplayers_progress_bar = create_progress_bar(&multi_progress, "NPlayers");
    let series_progress_bar = create_progress_bar(&multi_progress, "Series");
    let catver_progress_bar = create_progress_bar(&multi_progress, "Catver");

    let resources_handle = spawn_download(
        MameDataType::Resources,
        destination_folder.to_path_buf(),
        resources_progress_bar,
    );

    let nplayers_handle = spawn_download(
        MameDataType::NPlayers,
        destination_folder.to_path_buf(),
        nplayers_progress_bar,
    );

    let series_handle = spawn_download(
        MameDataType::Series,
        destination_folder.to_path_buf(),
        series_progress_bar,
    );

    let catver_handle = spawn_download(
        MameDataType::Catver,
        destination_folder.to_path_buf(),
        catver_progress_bar,
    );

    multi_progress.join().unwrap();

    let resources_result = resources_handle.join().unwrap();
    let nplayers_result = nplayers_handle.join().unwrap();
    let series_result = series_handle.join().unwrap();
    let catver_result = catver_handle.join().unwrap();

    if let Err(e) = resources_result {
        eprintln!("Error downloading Resources file: {}", e);
    }

    if let Err(e) = nplayers_result {
        eprintln!("Error downloading NPlayers file: {}", e);
    }

    if let Err(e) = series_result {
        eprintln!("Error downloading Series file: {}", e);
    }

    if let Err(e) = catver_result {
        eprintln!("Error downloading Catver file: {}", e);
    }

    Ok(())
}
