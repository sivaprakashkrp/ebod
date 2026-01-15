use std::{fs, io::Write, path::{Path, PathBuf}};

use crate::{dependencies::recursive_listing, log_deps::{LogType, log}, structs::FileEntry};

// A function to create metadata about the directory in .ebod/metadata.json
pub fn initialize_dir(path: &PathBuf, hidden_files: bool) {
    let mut data: Vec<FileEntry> = vec![];
    recursive_listing(&path, &path, &mut data, hidden_files);

    // pushing ".ebod/" into path
    let config_path = Path::new(path).join(".ebod");

    // deleting previous metadata.json file
    if let Ok(_success) = fs::remove_dir_all(&config_path) {
        log(LogType::Info, "Cleaned the directory of pre-existing ebod directories");
    } else {
        log(LogType::Err, "Error in deleting pre-exisiting directories");
    }

    // creating the directory
    if let Ok(_success) = fs::create_dir_all(&config_path) {
        log(LogType::Info, &format!("Ensured existance of all directories in {}", config_path.to_str().unwrap_or("default")));
    } else {
        log(LogType::Err, "Error Occurred during directory creation");
    }

    // adding metadata.json file to path
    let file_path = PathBuf::from(&config_path).join("metadata.json");

    // Converting data into JSON and writing it to the file
    if let Ok(data_string)= serde_json::to_string_pretty(&data) {
        if let Ok(mut file) = fs::File::create(&file_path) {
            if let Ok(_success) = file.write_all(data_string.as_bytes()) {
                log(LogType::Ok, &format!("Configuration files created at {}", file_path.to_str().unwrap_or("default")));
                if let Ok(_success) = hf::hide(PathBuf::from(&config_path)) {
                    log(LogType::Info, &format!("{} directory has been hidden", &config_path.to_str().unwrap_or("Path Couldn't be Unwraped")));
                } else {
                    log(LogType::Err, "Error in hiding the .ebod directory");
                }
            } else {
                log(LogType::Err, "Error occurred during writing data to the metadata.json");
            }
        } else {
            log(LogType::Err, "Error Occurred during creating the config file");
        }
    } else {
        log(LogType::Err, "Error during serializing data into toml");
    }
}