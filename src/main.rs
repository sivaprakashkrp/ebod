#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::{fs, io::Write, path::{Path, PathBuf}, time::{Duration, SystemTime, UNIX_EPOCH}};
use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct CLI {
    src: PathBuf,
    dest: Option<PathBuf>,
    #[arg(short='a', long="include-hidden")]
    include_hidden: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init {
        path: Option<PathBuf>,
    },
    Copy {
        src: PathBuf,
        dest: PathBuf,
    }
}

#[derive(Debug, Serialize)]
enum EntryType {
    Dir,
    File,
}

#[derive(Debug, Serialize)]
struct FileEntry {
    name: String,
    created_at: u64,
    modified_at: u64,
    length: u64,
    e_type: EntryType,
    #[cfg(target_os = "linux")]
    inode: u64,
    #[cfg(target_os = "windows")]
    file_attr: u32,
}

fn main() {
    let cli = CLI::parse();

    let src = cli.src;
    let dest = cli.dest.unwrap_or(PathBuf::from("."));

    initialize_dir(src, cli.include_hidden);
}

fn initialize_dir(path: PathBuf, hidden_files: bool) {
    let mut data: Vec<FileEntry> = vec![];
    recursive_listing(&path, &mut data, hidden_files);

    // pushing ".ebod/" into path
    let config_path = Path::new(&path).join(".ebod");

    // deleting previous metadata.json file
    if let Ok(success) = fs::remove_dir_all(&config_path) {
        println!("Cleaned the directory of pre-existing ebod directories");
    } else {
        println!("Error in deleting pre-exisiting directories");
    }

    // creating the directory
    if let Ok(success) = fs::create_dir_all(&config_path) {
        println!("Ensured all directories in {} exist", config_path.to_str().unwrap_or("default"));
    } else {
        println!("Error Occurred during directory creation");
    }

    // adding metadata.json file to path
    let file_path = PathBuf::from(&config_path).join("metadata.json");

    // Converting data into JSON and writing it to the file
    if let Ok(data_string)= serde_json::to_string_pretty(&data) {
        if let Ok(mut file) = fs::File::create(&file_path) {
            if let Ok(success) = file.write_all(data_string.as_bytes()) {
                println!("Configuration files created at {}", file_path.to_str().unwrap_or("default"));
                if let Ok(success) = hf::hide(PathBuf::from(config_path)) {
                    println!(".ebod directory has been hidden");
                } else {
                    println!("Error in hiding the .ebod directory");
                }
            } else {
                println!("Error occurred during writing data to the metadata.json");
            }
        } else {
            println!("Error Occurred during creating the config file");
        }
    } else {
        println!("Error during serializing data into toml");
    }
}

// A function to print the structure of the data recursively
fn recursive_listing(path: &PathBuf, data: &mut Vec<FileEntry>, include_hidden: bool) {
    if let Ok(read_dir) = fs::read_dir(&path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                if !include_hidden && file.file_name().into_string().unwrap_or("default".into()).starts_with(".") {
                    continue;
                }
                if let Ok(meta) = fs::metadata(file.path()) {
                    data.push(FileEntry {
                        name: PathBuf::from(path).join(PathBuf::from(file.file_name().into_string().unwrap_or("default".into()))).to_str().unwrap_or("default").to_string(),
                        created_at: if let Ok(create_time) = meta.created() {
                            create_time.duration_since(UNIX_EPOCH).expect("Error with SystemTime").as_secs()
                        } else {
                            0
                        },
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
                        recursive_listing(&file.path(), data, include_hidden);
                    }
                }
            }
        }
    }
}