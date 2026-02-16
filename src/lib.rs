use colored::Colorize;
use std::{fs::{self, create_dir_all}, path::PathBuf, process::exit};
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::{time::UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::{Path}};
use std::{fs::remove_file, io::ErrorKind};

/// Enum to store either the file entry is a `File` or a `Dir` (Directory).
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum EntryType {
    Dir,
    File,
}

/// The structure with which the metadata of the files are stored, retrieved and worked upon.
/// 
/// # Members
/// * `name: String` -> Stores the relative path of a file from the root directory where `ebod` is called.
/// * `modified_at: u64` -> The timestamp is seconds when the file or directory was last modified.
/// * `length: u64` -> The size of the file or directory in bytes
/// * `e_type: EntryType` -> The type of the entry. Either `EntryType::File` or `EntryType::Dir`.
/// * `inode: u64` **[LINUX ONLY]** -> Stores the Inode number of the file.
/// * `file_attr: u32` **[WINDOWS ONLY]** -> Stores the File attribute of the file.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct FileEntry {
    pub name: String,
    pub modified_at: u64,
    pub length: u64,
    pub e_type: EntryType,
    #[cfg(target_os = "linux")]
    pub inode: u64,
    #[cfg(target_os = "windows")]
    pub file_attr: u32,
}

/// A structure that holds the types of messages that can be displayed by the log function.
/// 
/// # Values in Enum
/// * `Info` -> Used when warnings or information messages are required to be displayed
/// * `Ok` -> Used to display success messages
/// * `Err` -> Used to display error messages
/// 
/// # Macros Applied
/// * `Debug`
/// * `PartialEq`
/// * `Eq`
#[derive(Debug, PartialEq, Eq)]
pub enum LogType {
    Info,
    Ok,
    Err,
}

/// Function with which all the progress is logged in `ebod`. The `LogType` has three options: `Info`, `Ok` and `Err`. `Info` is mapped to yellow, `Ok` to green and `Err` to red for efficient and easy to read logging.
/// 
/// # Input
/// * `logtype: LogType` -> The type of message that should be dispalyed
/// * `msg: &str` -> The message to be displayed
// Logging function for color-coded log messages
pub fn log(logtype: LogType, msg: &str) {
    if logtype == LogType::Info {
        println!("{}: {}", " INFO ".on_yellow().bold(), msg);
    } else if logtype == LogType::Ok {
        println!("{}: {}", "  OK  ".on_green().bold(), msg);
    } else if logtype == LogType::Err {
        println!("{}: {}", " ERR! ".on_red().bold(), msg.red());
    }
}

/// A function to initalize the directory for `ebod`. This is the function that is called when the `ebod init` command is executed. It creates the `./.ebod/metadata.json` file. Scans the directory for information of files and loads the metadata into `metadata.json`. 
/// 
/// The function also hides the `./.ebod` directory.
/// 
/// # Input
/// * `path: &PathBuf` -> The path of the directory in which `ebod` should be initialized
/// * include_hidden: bool` -> The boolean flag which tells whether to include or exclude hidden files
// A function to create metadata about the directory in .ebod/metadata.json
pub fn initialize_dir(path: &PathBuf, include_hidden: bool) {
    let mut data: Vec<FileEntry> = vec![];
    recursive_listing(&path, &path, &mut data, include_hidden);

    // pushing ".ebod/" into path
    let config_path = Path::new(path).join(".ebod");

    // deleting previous metadata.json file if the .ebod folder exists
    if config_path.exists() {
        if let Ok(_success) = fs::remove_dir_all(&config_path) {
            log(LogType::Info, "Cleaned the pre-existing .ebod directories");
        } else {
            log(LogType::Err, "Error in deleting pre-exisiting .ebod directories");
        }
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
                if !hf::is_hidden(file_path).unwrap_or(false) {
                    if let Ok(_success) = hf::hide(PathBuf::from(&config_path)) {
                        log(LogType::Info, &format!("{} directory has been hidden", &config_path.to_str().unwrap_or("Path Couldn't be Unwraped")));
                    } else {
                        log(LogType::Err, "Error in hiding the .ebod directory");
                    }
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

/// Backs up data present in the src folder into the dest folder.
/// 
/// # Inputs
/// * `src: &PathBuf` -> `&PathBuf` of the source directory
/// * `dest: &PathBuf` -> `&PathBuf` of the destination directory
/// 
/// # Output: `Result<(), String>`
/// 
/// The function first calls `read_metadata` function on both the src and dest directories. Then the metadata recorded is stored in the `metadata.json` file inside the hidden folder `.ebod`.
/// 
/// Then based on the file names and their metadata present in the metadata, the files are copied from src to dest.
/// 
/// # Rules followed:
/// 1. A file in the src is checked for its existence in the dest by checking all of its stored metadata. If the file exists, then it is not copied.
/// 2. If there is a file in src and dest with the same name, then the `modified_at` attribute of the files are checked. If they are equal then the file is not copied.
/// 3. If not, then the file from src is copied to dest with the filename `ebod-src-filename`. The user is prompted to change the file name at the end of the Backup process
/// 
// Backup the files in the src directory in to the dest directory
pub fn backup(src: &PathBuf, dest: &PathBuf, dir :&str) -> Result<(), String> {
    let src_path = src.join(PathBuf::from(".ebod/metadata.json"));
    let dest_path = dest.join(PathBuf::from(".ebod/metadata.json"));

    let mut redundant_files: Vec<FileEntry> = vec![];
    let mut copied_files_with_new_name: Vec<String> = vec![];

    let src_meta = read_metadata(&src_path).unwrap_or(vec![]);
    let dest_meta = read_metadata(&dest_path).unwrap_or(vec![]);

    for file in src_meta {
        if dest_meta.contains(&file) {
            redundant_files.push(file.clone());
            continue;
        } else if file.e_type == EntryType::Dir && ! dest_meta.contains(&file) {
            if let Err(_error) = create_dir_all(&dest.join(&file.name)) {
                return Err(format!("Couldn't create directory {}", &file.name));
            } else {
                log(LogType::Ok, &format!("Created Directory: {} in destination", &file.name));
            }
        } else {
            let idx = check_with_filename(&file.name, &dest_meta);
            if idx != -1 {
                if dest_meta.get(idx as usize).unwrap().modified_at != file.modified_at {
                    let redundant_file_name = rename_redundant_files(&file.name, dir);
                    if let Err(_error) = copy_file(&src.join(PathBuf::from(&file.name)), &dest.join(PathBuf::from(&redundant_file_name))) {
                        return Err(format!("Error copying file: {}", &file.name));
                    } else {
                        log(LogType::Ok, &format!("Copied file: {} to destination", &file.name));
                    }
                    log(LogType::Info, &format!("{} found in destination is with varied modified time than {} in source. Hence it is copied under the name {}", file.name, dest_meta.get(idx as usize).unwrap().name, redundant_file_name));
                    copied_files_with_new_name.push(redundant_file_name);
                } else {
                    redundant_files.push(file.clone());
                }
                continue;
            }
            if let Err(_error) = copy_file(&src.join(PathBuf::from(&file.name)), &dest.join(PathBuf::from(&file.name))) {
                return Err(format!("Error copying file: {}", &file.name));
            } else {
                log(LogType::Ok, &format!("Copied file: {} to destination", &file.name));
            }
        }
    }

    if redundant_files.len() > 0 {
        log(LogType::Info, "Files that were present in both source and destination and hence were not copied:");
        for file in redundant_files {
            println!("\t{}", file.name.yellow());
        }
    }

    if copied_files_with_new_name.len() > 0 {
        log(LogType::Info, "Files that were present in both source and destination and hence were copied with new name:");
        println!("{}", "  -- Please change the names of the below files ASAP -- ".on_red().bold());
        for file in copied_files_with_new_name {
            println!("\t{}", file.yellow());
        }
    }

    Ok(())
}


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

/// The function that is called when `ebod sync` command is invoked. It syncs the files in the source and the destination directory. It works by first backing up the `src` into `dest`. Then `dest` is initialized again and then this `dist` is backed up into the `src` directory.
/// Then the copies in the `src` directory are cleared to avoid duplication.
/// 
/// The deletion of duplicates in `src` happens by selecting files that start with `ebod-src-` as these file names are generated during the back up process to prevent overwriting of already existing files.
/// 
/// # Input
/// * `src: &PathBuf` -> The `PathBuf` to the source directory
/// * `dest: &PathBuf` -> The `PathBuf` to the destination directory
/// * `include_hidden: bool` -> The boolean flag to represent inclusion of hidden files for synchronization process.
pub fn sync_dirs(src: &PathBuf, dest: &PathBuf, include_hidden: bool) {
    if let Ok(_suc) = backup(&src, &dest, "src") {
        log(LogType::Ok, &format!("{} was backed up into {}", src.to_string_lossy(), dest.to_string_lossy()));
        initialize_dir(&dest, include_hidden);
        if let Ok(_success) = backup(&dest, &src, "dest") {
            log(LogType::Ok, &format!("{} was backed up into {}", dest.to_string_lossy(), src.to_string_lossy()));
        } else {
            log(LogType::Err, &format!("There was an error in backing up {} into {} dirctory", dest.to_string_lossy(), src.to_string_lossy()));
        }
        let mut src_meta = vec![];
        recursive_listing(&src, &src, &mut src_meta, include_hidden);
        if let Ok(_suc) = delete_copies_in_dir(&src, &src_meta) {
            log(LogType::Ok, &format!("Duplicate files in {} have been successfully deleted.", src.to_string_lossy()));
        } else {
            log(LogType::Err, &format!("Error in deleting duplicate files in the {} directory.", src.to_string_lossy()));
        }
    } else {
        log(LogType::Err, &format!("There was an error in backing up {} into {} dirctory", src.to_string_lossy(), dest.to_string_lossy()));
    }
}

/// The function deletes duplicates files in the directory passed as input. Useful to clean-up duplicate files after syncing two directories.
/// 
/// # Input
/// * `src: &PathBuf` -> The `PathBuf` to the directory in which the duplicates must be deleted.
/// * `src_meta: &Vec<FileEntry>` -> The metadata that we get from calling the function `initialize_dir()`
pub fn delete_copies_in_dir(src: &PathBuf, src_meta: &Vec<FileEntry>) -> Result<(), String> {
    let mut error: bool = false;
    for file in src_meta {
        let file_pathbuf = PathBuf::from(&file.name);
        if let Some(file_name_os) = file_pathbuf.file_name() {
            if let Some(file_name) = file_name_os.to_str() {
                if file_name.starts_with("ebod-src-") {
                    match remove_file(&src.join(PathBuf::from(&file.name))) {
                        Ok(()) => log(LogType::Ok, &format!("File removed successfully: {}", &file.name)),
                        Err(e) => {
                            match e.kind() {
                                ErrorKind::NotFound => log(LogType::Err,&format!("Error: File not found at {}", &file.name)),
                                ErrorKind::PermissionDenied => log(LogType::Err,&format!("Error: Permission denied to remove file at {}", &file.name)),
                                _ => log(LogType::Err,&format!("Error removing file {}: {:?}", &file.name, e)),
                            }
                            error = true;
                        }
                    }
                }
            } else {
                log(LogType::Err, &format!("Error in resolving the file name of {}", &file.name));
                error = true;
            }
        } else {
            log(LogType::Err, &format!("Error in extracting file name from pathbuf for {}", &file.name));
            error = true;
        }
    }
    if error {
        return Err(String::from("Error Occurred"));
    }
    Ok(())
}

pub fn check_dir_existence(dir: &PathBuf) {
    if !dir.exists() {
        log(LogType::Err, &format!("The directory {} doesn't exist!!", dir.to_string_lossy()));
        exit(0);
    }
}