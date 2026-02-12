#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::{fs, path::PathBuf, time::UNIX_EPOCH};

use crate::{log_deps::{LogType, log}, structs::{EntryType, FileEntry}};

/// Checks if the filename in src already exists in the dest directory
/// 
/// # Inputs
/// * `file: &String` -> `&String` which contains the file name
/// * `dest_meta: Vec<FileEntry>` -> A `Vec<FileEntry>` that contains the metatdata of the dest directory
/// 
/// # Output: `i16`
/// The index of the file in the destination metadata as a `i16` 
pub fn check_with_filename(file: &String, dest_meta: &Vec<FileEntry>) -> i16 {
    for (index, entry) in dest_meta.iter().enumerate() {
        if entry.name == *file {
            return index as i16;
        }
    }
    -1
}

fn _rename_redundant_files_at_end(file: &str) -> String {
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

/// Renaming redundant files to prevent overwriting
/// 
/// # Inputs
/// * `file: &str` -> A string slice with the file name
/// * `dir: &str` -> A string slice with the directory name (src/dest)
/// 
/// # Output: `String` 
/// A String with the newly created file name to replace the redundant name.
pub fn rename_redundant_files(file: &str, dir: &str) -> String {
    if let Some(file_name) = PathBuf::from(file).file_name() {
        let old_file_name = String::from(file_name.to_str().unwrap_or("default"));
        return format!("ebod-{}-{}", dir, old_file_name);
    } else {
        log(LogType::Err, &format!("Couldn't resolve the file name of {}. Stored as ebod-src-default", file));
        return String::from("ebod-src-default");
    }
}

/// Abstraction for the file copying mechanism
/// 
/// # Inputs
/// * `src: &PathBuf` -> `&PathBuf` of the source file
/// * `dest: &PathBuf` -> `&PathBuf` of the destination file
/// 
/// # Output: `Result<u64, String>`
/// A `Result<u64, String>` with an error message in the string in case of errors.
pub fn copy_file(src: &PathBuf, dest: &PathBuf) -> Result<u64, String> {
    if let Ok(_success) = fs::copy(src, dest) {
        Ok(_success)
    } else {
        Err(String::from(format!("Error in copying file {}", src.to_str().unwrap())))
    }
}

/// Abstraction for the mechanism that reads the metadata from `.ebod/metadata.json` and returns it
/// 
/// # Input
/// * `path` -> `&PathBuf` of the metadata file
/// 
/// # Output: `Result<Vec<FileEntry, String>`
/// A `Result<Vec<FileEntry, String>` that either has the stored metadata or error message
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

/// A function to traverse the directories and files recursively and store their metadata.
/// 
/// # Input
/// * `path: &PathBuf` -> `&PathBuf` of the directory whose metadata is required
/// * `og_path: &PathBuf` -> Same `&PathBuf` as `path`. Used to prefix the directory name in each file in metadata
/// * `data: &mut Vec<FileEntry>` -> `Vec<FileEntry>` which is the buffer in which the data is recorded.
/// * `include_hidden: bool` -> `bool` flag to represent the inclusion of hidden files
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

