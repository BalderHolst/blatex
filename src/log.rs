use std::{path::PathBuf, process::exit};

pub fn print_log(main_file: &str) {
    let log_file = PathBuf::from(main_file).with_extension("log");

    if !log_file.is_file() {
        eprintln!("Cannot find log file `{}`.", log_file.display());
        exit(1);
    }

    let log = texlog::log::Log::from_path(log_file);
    log.print_diagnostics();
}
