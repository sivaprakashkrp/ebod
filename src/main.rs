use std::path::PathBuf;
use clap::{Parser, Subcommand};

// Importing from dependencies.rs
mod dependencies;
use crate::dependencies::{LogType, backup, initialize_dir, log};


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
        #[arg(short='a', long="include-hidden")]
        include_hidden: bool,
    },
    Sync {
        src: PathBuf,
        dest: PathBuf,
    }
}

fn main() {
    let cli = CLI::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Init { path, include_hidden } => {
                initialize_dir(&path.unwrap_or(PathBuf::from(".")), include_hidden);
            }
            Commands::Sync { src, dest } => {
                // Need to implement Syncing procedure
            }
        }
    } else {
        let dest = cli.dest.unwrap_or(PathBuf::from("."));
        copy_src_into_dest(cli.src, dest, cli.include_hidden);
    }
}

fn copy_src_into_dest(src: PathBuf, dest: PathBuf, include_hidden: bool) {
    // Initializing the .ebod directories in both the folders
    initialize_dir(&src, include_hidden);
    initialize_dir(&dest, include_hidden);
    
    if let Err(err) = backup(&src, &dest) {
        log(LogType::Err, &format!("{}", err));
    }
}