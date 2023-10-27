use std::{path::PathBuf, process::exit};

pub fn replace_text(s: String, pattern: &str, value: &str) -> String {
    let (first, second) = s.split_once(pattern).expect("pattern not found.");
    first.to_string() + value + second
}

/// Get current working directory or panic
pub fn get_cwd() -> PathBuf {
    match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    }
}
