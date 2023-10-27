use termion::color::{self, Fg};

use crate::{log, utils, opts::Config};

pub fn compile(config: Config, main_file: &String) {
    let cmd = utils::replace_text(&config.compile_cmd, "<main-file>", main_file.as_str());

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
            }
        }
        None => {
            eprintln!("Compilation process stopped unexpectedly");
            std::process::exit(1)
        }
    };

    // Parse log file
    log::print_log(main_file.as_str());
}
