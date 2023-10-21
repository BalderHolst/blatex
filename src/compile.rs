use termion::color::{self, Fg};

use crate::utils;

pub fn compile(compile_cmd: String, main_file: String) {
    let cmd = utils::replace_text(compile_cmd, "<main-file>", main_file.as_str());

    println!(
        "{}Running command: `{}`{}\n",
        Fg(color::Blue),
        cmd,
        Fg(color::Reset)
    );

    let status = if cfg!(target_os = "windows") {
        eprintln!("Compilation on windows is currently not supported.");
        std::process::exit(1);
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .unwrap()
    };

    match status.code() {
        Some(code) => {
            if code != 0 {
                eprintln!(
                    "\n{}Compilation process exited with non-zero exit code: {}{}",
                    Fg(color::Red),
                    code,
                    Fg(color::Reset)
                );
                std::process::exit(1)
            }
        }
        None => {
            eprintln!("Compilation process stopped unexpectedly");
            std::process::exit(1)
        }
    };
}
