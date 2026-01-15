use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum EntryType {
    Dir,
    File,
}

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