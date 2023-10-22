use std::path::PathBuf;

pub fn print_log(main_file: String) {
    let log_file = PathBuf::from(main_file).with_extension("log");
    let log = texlog::log::Log::from_path(log_file);
    log.print_diagnostics();
}
