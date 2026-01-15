use std::{fs::create_dir_all, path::PathBuf};

use colored::Colorize;

use crate::{dependencies::{check_with_filename, copy_file, read_metadata, rename_redundant_files}, log_deps::{LogType, log}, structs::{EntryType, FileEntry}};

// Backup the files in the src directory in to the dest directory
pub fn backup(src: &PathBuf, dest: &PathBuf) -> Result<(), String> {
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
                    let redundant_file_name = rename_redundant_files(&file.name);
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
        for file in copied_files_with_new_name {
            println!("\t{}", file.yellow());
        }
        println!("{}", "  -- Please change the names of the above files ASAP -- ".on_red().bold())
    }

    Ok(())
}