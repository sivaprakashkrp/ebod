# `ebod`
Easy/Efficient Backup Of Data

A command-line utility that makes the process of manual backups easy and efficient

## The `init` subcommand

The `init` subcommand is used to initialise a directory so that `ebod` can effieciently transfer the data in the directory. 

The `init` subcommand works by recursively traversing the directory structure of the input path and 

### Arguments

```bash
ebod init [PATH] [OPTIONS]
```

### Options
`[PATH]` -> The path of the directory in which `ebod` must be initialized. By default it is the current directory, ".".

`-a` or `--include-hidden` -> Tell `ebod` to include hidden files while traversing the directory.
