use std::{fs::remove_file, io::ErrorKind, path::PathBuf};

use crate::{backup_deps::backup, dependencies::recursive_listing, init_deps::initialize_dir, log_deps::{LogType, log}, structs::FileEntry};

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
        if let Ok(_suc) = delete_copies_in_src(&src, &src_meta) {
            log(LogType::Ok, &format!("Duplicate files in {} have been successfully deleted.", src.to_string_lossy()));
        } else {
            log(LogType::Err, &format!("Error in deleting duplicate files in the {} directory.", src.to_string_lossy()));
        }
    } else {
        log(LogType::Err, &format!("There was an error in backing up {} into {} dirctory", src.to_string_lossy(), dest.to_string_lossy()));
    }
}

fn delete_copies_in_src(src: &PathBuf, src_meta: &Vec<FileEntry>) -> Result<(), String> {
    let mut error: bool = false;
    for file in src_meta {
        // println!("{:?}", file);
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