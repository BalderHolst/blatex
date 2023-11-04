use std::path::PathBuf;

use termion::color::{self, Fg};

use crate::{
    opts::{CleanArgs, Config},
    utils,
};

pub fn clean(cwd: PathBuf, config: Config, args: CleanArgs) {
    let main_file = &args.main_file.unwrap_or(config.main_file.clone());

    let cmd = utils::replace_text(&config.clean_cmd, "<main-file>", main_file.as_str());
    let prefix = format!("cd \"{}\"", cwd.display());

    let cmd = prefix + " && " + cmd.as_str();

    println!(
        "{}Running command: `{}`{}\n",
        Fg(color::Blue),
        cmd,
        Fg(color::Reset)
    );

    let status = if cfg!(target_os = "windows") {
        eprintln!("Cleaning on windows is currently not supported.");
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
                    "\n{}Cleaning process exited with non-zero exit code: {}{}",
                    Fg(color::Red),
                    code,
                    Fg(color::Reset)
                );
                std::process::exit(1)
            }
        }
        None => {
            eprintln!("Cleaning process stopped unexpectedly");
            std::process::exit(1)
        }
    };
}
