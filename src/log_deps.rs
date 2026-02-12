use colored::Colorize;

/// A structure that holds the types of messages that can be displayed by the log function.
/// 
/// # Values in Enum
/// * `Info` -> Used when warnings or information messages are required to be displayed
/// * `Ok` -> Used to display success messages
/// * `Err` -> Used to display error messages
/// 
/// # Macros Applied
/// * `Debug`
/// * `PartialEq`
/// * `Eq`
#[derive(Debug, PartialEq, Eq)]
pub enum LogType {
    Info,
    Ok,
    Err,
}

/// Function with which all the progress is logged in `ebod`. The `LogType` has three options: `Info`, `Ok` and `Err`. `Info` is mapped to yellow, `Ok` to green and `Err` to red for efficient and easy to read logging.
/// 
/// # Input
/// * `logtype: LogType` -> The type of message that should be dispalyed
/// * `msg: &str` -> The message to be displayed
// Logging function for color-coded log messages
pub fn log(logtype: LogType, msg: &str) {
    if logtype == LogType::Info {
        println!("{}: {}", " INFO ".on_yellow().bold(), msg);
    } else if logtype == LogType::Ok {
        println!("{}: {}", "  OK  ".on_green().bold(), msg);
    } else if logtype == LogType::Err {
        println!("{}: {}", " ERR! ".on_red().bold(), msg.red());
    }
}