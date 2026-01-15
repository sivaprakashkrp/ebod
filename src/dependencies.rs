#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::{fs, path::PathBuf, time::UNIX_EPOCH};

use crate::{log_deps::{LogType, log}, structs::{EntryType, FileEntry}};

// Check if the filename in src already exists in the dest directory
pub fn check_with_filename(file: &String, dest_meta: &Vec<FileEntry>) -> i16 {
    for (index, entry) in dest_meta.iter().enumerate() {
        if entry.name == *file {
            return index as i16;
        }
    }
    -1
}

// Renaming redundant files to prevent overwriting
pub fn rename_redundant_files(file: &str) -> String {
    let mut new_file_name = String::from("");

    if let Some(file_name) = PathBuf::from(file).file_name() {
        let old_file_name = String::from(file_name.to_str().unwrap_or("default"));
        new_file_name.push_str(&old_file_name[..old_file_name.rfind(".").unwrap_or(4)]);
        new_file_name.push_str("-src-copy.")
    } else {
        log(LogType::Err, &format!("Couldn't resolve the file name of {}", file));
    }

    if let Some(extension) =  PathBuf::from(file).extension() {
        new_file_name.push_str(extension.to_str().unwrap_or("default"));
    } else {
        log(LogType::Err, &format!("Couldn't resolve the extension of {}", file));
    }

    new_file_name
}

// Abstraction for the file copying mechanism
pub fn copy_file(src: &PathBuf, dest: &PathBuf) -> Result<u64, String> {
    if let Ok(_success) = fs::copy(src, dest) {
        Ok(_success)
    } else {
        Err(String::from(format!("Error in copying file {}", src.to_str().unwrap())))
    }
}

// Abstraction for the mechanism that parses the directory, its files and records metadata
pub fn read_metadata(path: &PathBuf) -> Result<Vec<FileEntry>, String> {
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
pub fn recursive_listing(path: &PathBuf, og_path: &PathBuf, data: &mut Vec<FileEntry>, include_hidden: bool) {
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

