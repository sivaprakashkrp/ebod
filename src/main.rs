use std::path::PathBuf;
use clap::{Parser, Subcommand};

// Importing from lib.rs
use ebod::{LogType, backup, check_dir_existence, initialize_dir, log, sync_dirs};


#[derive(Parser, Debug)]
#[command(
    version,
    author,
    about = "Easy/Efficient Backup Of Data",
    long_about = "An efficient backup solution developed in Rust. Can be used to make manual backups easy and efficient.",
    help_template = "{bin} {version}\nDeveloped By: {author}\n\n{about}\n\nUsage:\n\t{usage}\n\n{all-args}",
    author = "Sivaprakash P"
)]
struct CLI {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        version,
        author,
        about = "Used to initalize a directory for ebod.",
        long_about="Reads the directory and stores the metadata about files for efficient backup. Creates the .ebod directory(hidden) with metadata.json inside it.",
        help_template = "{bin} {version}\nDeveloped By: {author}\n\n{about}\n\nUsage:\n\t{usage}\n\n{all-args}",
        author = "Sivaprakash P"
    )]
    Init {
        #[arg(help="Relative path to the Directory to be initialized (Default = '.')")]
        path: Option<PathBuf>,
        #[arg(short='a', long="include-hidden", help="Includes the hidden files and directories in the input directories")]
        include_hidden: bool,
    },
    #[command(
        version,
        author,
        about = "Used to Sync the Source and Destination directories.",
        long_about="Used to sync the Source and Destination directories. Both files from the Source directory and from the Destination directory are copied into each other.",
        help_template = "{bin} {version}\nDeveloped By: {author}\n\n{about}\n\nUsage:\n\t{usage}\n\n{all-args}",
        author = "Sivaprakash P"
    )]
    Sync {
        src: PathBuf,
        dest: Option<PathBuf>,
        #[arg(short='a', long="include-hidden", help="Includes the hidden files and directories in the input directories")]
        include_hidden: bool
    },
    #[command(
        version,
        author,
        about = "Used to backup the source directory into the destination directory", long_about="Copies the missing files from the source into destination. If files with same name was found, then checks for modified_at timestamp. If the timestamp is equal, then the file is not copied. If not, then the file is coped under the name \"<old_file_name>-src-copy.<extension>\". Please make sure to rename such files as soon as possible.",
        help_template = "{bin} {version}\nDeveloped By: {author}\n\n{about}\n\nUsage:\n\t{usage}\n\n{all-args}",
        author = "Sivaprakash P"
    )]
    Backup {
        #[arg(help="Relative path to Source Directory")]
        src: PathBuf,
        #[arg(help="Relative path to Destination Directory")]
        dest: Option<PathBuf>,
        #[arg(short='a', long="include-hidden", help="Includes the hidden files and directories in the Source and Destination directory")]
        include_hidden: bool,
    }
}

fn main() {
    let cli = CLI::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Init { path, include_hidden } => {
                check_dir_existence(&path.clone().unwrap_or(PathBuf::from(".")));
                initialize_dir(&path.unwrap_or(PathBuf::from(".")), include_hidden);
            },
            Commands::Sync { src, dest, include_hidden } => {
                check_dir_existence(&src);
                let dest_path = dest.unwrap_or(PathBuf::from("."));
                initialize_dir(&src, include_hidden);
                initialize_dir(&dest_path, include_hidden);
                sync_dirs(&src, &dest_path, include_hidden);
            },
            Commands::Backup { src, dest, include_hidden } => {
                check_dir_existence(&src);
                let dest = dest.unwrap_or(PathBuf::from("."));
                copy_src_into_dest(src, dest, include_hidden);
            },
        }
    }
}

/// Abstracted function to copy `src` into `dest`
/// 
/// # Input
/// * `src: &PathBuf` -> The `PathBuf` to the source directory
/// * `dest: &PathBuf` -> The `PathBuf` to the destination directory
/// * `include_hidden: bool` -> The boolean flag to represent inclusion of hidden files for backup process.
fn copy_src_into_dest(src: PathBuf, dest: PathBuf, include_hidden: bool) {
    // Initializing the .ebod directories in both the folders
    initialize_dir(&src, include_hidden);
    initialize_dir(&dest, include_hidden);
    
    if let Err(err) = backup(&src, &dest, "src") {
        log(LogType::Err, &format!("{}", err));
    }
}