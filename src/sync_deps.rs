use std::path::PathBuf;

use crate::{backup_deps::backup, init_deps::initialize_dir, log_deps::{LogType, log}};

pub fn sync_dirs(src: &PathBuf, dest: &PathBuf, include_hidden: bool) {
    if let Ok(suc) = backup(&src, &dest) {
        log(LogType::Ok, &format!("{} was backed up into {}", src.to_string_lossy(), dest.to_string_lossy()));
        initialize_dir(&dest, include_hidden);
        if let Ok(success) = backup(&dest, &src) {
            log(LogType::Ok, &format!("{} was backed up into {}", dest.to_string_lossy(), src.to_string_lossy()));
        } else {
            log(LogType::Err, &format!("There was an error in backing up {} into {} dirctory", dest.to_string_lossy(), src.to_string_lossy()));
        }
    } else {
        log(LogType::Err, &format!("There was an error in backing up {} into {} dirctory", src.to_string_lossy(), dest.to_string_lossy()));
    }
}