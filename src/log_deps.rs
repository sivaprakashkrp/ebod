use colored::Colorize;

#[derive(Debug, PartialEq, Eq)]
pub enum LogType {
    Info,
    Ok,
    Err,
}

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