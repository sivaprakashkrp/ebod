use serde::{Deserialize, Serialize};

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