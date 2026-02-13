# `ebod`

[![Crates.io](https://img.shields.io/crates/v/ebod.svg)](https://crates.io/crates/ebod)

Easy/Efficient Backup Of Data

A command-line utility that makes the process of manual backups easy and efficient.

While taking local backups, we either have to over-write a folder that we have already backed up and would like to add some more file to. This has to be done as there might be already existing files, files with same names in both source and destination directories but having different content, i.e., different modified times. 

`ebod` is a command-line tool that can be used in such scenarios. It backs up data smartly and efficiently by reading the directory structure and backing up just the files that are required to be backed up. You can even sync your source and destination directories with the `sync` subcommand. 

`ebod` was created as a easy way to create local backups. A naive implementation of the backing up and syncing mechanisms have been created.

## Installation

### Using `cargo`

You can now install [`ebod`](https://crates.io/crates/ebod) through [`cargo`](https://crates.io/) with the command:
```bash
cargo install ebod
```

### For Windows Systems

If you are on a Windows (`x86_64`) system, then you can refer to the build binary in **Releases** section. Download it and add the path of the parent folder to `PATH` variable.

### Building from source

You can build `ebod` from source by cloning this directory, `cd` into the repository and running 
```bash
cargo build --release
```
Then you just have to add the path of the application, usually in `<path-to-repo>/target/release/`, to your system's `PATH` variable.

## The `init` subcommand

The `init` subcommand is used to initialise a directory so that `ebod` can effieciently transfer the data in the directory. 

The `init` subcommand works by recursively traversing the directory structure of the input path and 

### Arguments

```bash
ebod init [PATH] [OPTIONS]
```

### Options
`[PATH]` -> The path of the directory in which `ebod` must be initialized. By default it is the current directory, ".".

### Include Hidden files and Directories
```
-a, --include-hidden
```
Tell `ebod` to include hidden files while traversing the directory.

## The `backup` subcommand

The `backup` command is used to backup files from the `src` directory into the `dest` directory.

It first initializes both the directories and uses the generated metadata to efficiently backup files.

### Arguments

```bash
ebod backup <SRC> [DEST] [OPTIONS]
```

### Options
`<SRC>` -> The source directory.

`[DEST]` -> The destination directory, by default it is the current directory (".")

### Include Hidden files and Directories
```
-a, --include-hidden
```
Tell `ebod` to include hidden files while traversing the directory.


## The `sync` subcommand

The `sync` subcommand is used to sync the directories in the input paths by copying the latest files and directories from each of them into the other.

The `sync` command works by calling the `init` command first on both the `src` and `dest` directories, then backing up `src` into `dest`. Then `dest` is re-initialized, and the files in `dest` are backed up into `src`.

### Arguments

```bash
ebod sync <SRC> [DEST] [OPTIONS]
```

### Options
`<SRC>` -> The source directory.

`[DEST]` -> The destination directory, by default it is the current directory (".")

### Include Hidden files and Directories
```
-a, --include-hidden
```
Tell `ebod` to include hidden files while traversing the directory.

---

For more information and documentation, visit [docs.rs](https://docs.rs/crate/ebod/1.0.0/source/)

    