#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::{fs::{self, create_dir_all}, io::Write, path::{Path, PathBuf}, time::UNIX_EPOCH};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum LogType {
    Info,
    Ok,
    Err,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryType {
    Dir,
    File,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    name: String,
    modified_at: u64,
    length: u64,
    e_type: EntryType,
    #[cfg(target_os = "linux")]
    inode: u64,
    #[cfg(target_os = "windows")]
    file_attr: u32,
}

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
        log(LogType::Info, &format!("Ensured all directories in {} exist", config_path.to_str().unwrap_or("default")));
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
                if let Ok(_success) = hf::hide(PathBuf::from(config_path)) {
                    log(LogType::Ok, ".ebod directory has been hidden");
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

pub fn backup(src: &PathBuf, dest: &PathBuf) -> Result<(), String> {
    let src_path = src.join(PathBuf::from(".ebod/metadata.json"));
    let dest_path = dest.join(PathBuf::from(".ebod/metadata.json"));

    let src_meta = read_metadata(&src_path).unwrap_or(vec![]);
    let dest_meta = read_metadata(&dest_path).unwrap_or(vec![]);

    for file in src_meta {
        if dest_meta.contains(&file) {
            continue;
        } else if file.e_type == EntryType::Dir && ! dest_meta.contains(&file) {
            if let Err(_error) = create_dir_all(&dest.join(&file.name)) {
                return Err(format!("Couldn't create directory {}", &file.name));
            } else {
                log(LogType::Ok, &format!("Created Directory: {}", &file.name));
            }
        } else {
            if let Err(_error) = copy_file(&src.join(PathBuf::from(&file.name)), &dest.join(PathBuf::from(&file.name))) {
                return Err(format!("Error copying file: {}", &file.name));
            } else {
                log(LogType::Ok, &format!("Copied file: {}", &file.name));
            }
        }
    }

    Ok(())
}

fn copy_file(src: &PathBuf, dest: &PathBuf) -> Result<u64, String> {
    if let Ok(_success) = fs::copy(src, dest) {
        Ok(_success)
    } else {
        Err(String::from(format!("Error in copying file {}", src.to_str().unwrap())))
    }
}

fn read_metadata(path: &PathBuf) -> Result<Vec<FileEntry>, String> {
    if let Ok(file_content) = fs::read_to_string(path) {
        if let Ok(json_content) = serde_json::from_str(&file_content) {
            Ok(json_content)
        } else {
            Err(String::from("Error in parsing JSON data"))
        }
    } else {
        Err(String::from("Error in reading from file"))
    }
} 

// A function to print the structure of the data recursively
fn recursive_listing(path: &PathBuf, og_path: &PathBuf, data: &mut Vec<FileEntry>, include_hidden: bool) {
    if let Ok(read_dir) = fs::read_dir(&path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                if !include_hidden && file.file_name().into_string().unwrap_or("default".into()).starts_with(".") {
                    continue;
                }
                if let Ok(meta) = fs::metadata(file.path()) {
                    data.push(FileEntry {
                        name: file.path().strip_prefix(og_path).unwrap_or(file.path().as_path()).to_str().unwrap_or("default").to_string(),
                        modified_at: if let Ok(mod_time) = meta.modified() {
                            mod_time.duration_since(UNIX_EPOCH).expect("Error with SystemTime").as_secs()
                        } else {
                            0
                        },
                        length: meta.len(),
                        e_type: if meta.is_dir() {EntryType::Dir} else {EntryType::File},
                        #[cfg(target_os = "linux")]
                        inode: meta.ino(),
                        #[cfg(target_os = "windows")]
                        file_attr: meta.file_attributes()
                    });
                    if meta.is_dir() {
                        recursive_listing(&file.path(), og_path, data, include_hidden);
                    }
                }
            }
        }
    }
}

pub fn log(logtype: LogType, msg: &str) {
    if logtype == LogType::Info {
        println!("{}: {}", " INFO ".on_yellow().bold(), msg);
    } else if logtype == LogType::Ok {
        println!("{}: {}", "  OK  ".on_green().bold(), msg);
    } else if logtype == LogType::Err {
        println!("{}: {}", " ERR! ".on_red().bold(), msg.red());
    }
}