use std::{fs::create_dir_all, path::PathBuf};

use colored::Colorize;

use crate::{dependencies::{check_with_filename, copy_file, read_metadata, rename_redundant_files}, log_deps::{LogType, log}, structs::{EntryType, FileEntry}};

/// Backs up data present in the src folder into the dest folder.
/// 
/// # Inputs:
/// * `src` -> `&PathBuf` of the source directory
/// * `dest` -> `&PathBuf` of the destination directory
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
/// 3. If not, then the file from src is copied to dest with the filename "ebod-src-<filename>". The user is prompted to change the file name at the end of the Backup process
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