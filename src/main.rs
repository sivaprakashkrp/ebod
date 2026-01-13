use std::path::PathBuf;
use clap::{Parser, Subcommand};

// Importing from dependencies.rs
use crate::dependencies::{LogType, backup, initialize_dir, log};

mod dependencies;

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

fn main() {
    let cli = CLI::parse();

    let src = cli.src;
    let dest = cli.dest.unwrap_or(PathBuf::from("."));

    // Initializing the .ebod directories in both the folders
    initialize_dir(&src, cli.include_hidden);
    initialize_dir(&dest, cli.include_hidden);
    
    if let Err(err) = backup(&src, &dest) {
        log(LogType::Err, &format!("{}", err));
    }
}