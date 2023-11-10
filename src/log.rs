use std::path::PathBuf;

use crate::exit_with_error;

pub fn print_log(root: PathBuf, main_file: &PathBuf) {
    let log_file = root.join(main_file).with_extension("log");

    if !log_file.is_file() {
        exit_with_error!("Cannot find log file `{}`.", log_file.display());
    }

    let log = texlog::log::Log::from_path(log_file);
    log.print_diagnostics();
}
