use std::path::PathBuf;

use termion::color::{self, Fg};

use crate::{
    exit_with_error,
    opts::{CleanArgs, Config},
    utils,
};

pub fn clean(config: Config, args: CleanArgs) {
    let main_file = match args.main_file {
        Some(s) => PathBuf::from(s),
        None => config.main_file,
    };

    let cmd = utils::replace_path_placeholders(&config.clean_cmd, main_file.as_path());
    let prefix = format!("cd \"{}\"", config.root.display());

    let cmd = prefix + " && " + cmd.as_str();

    println!(
        "{}Running command: `{}`{}\n",
        Fg(color::Blue),
        cmd,
        Fg(color::Reset)
    );

    let status = if cfg!(target_os = "windows") {
        exit_with_error!("Cleaning on windows is currently not supported.");
    } else {
        match std::process::Command::new("sh").arg("-c").arg(cmd).status() {
            Ok(s) => s,
            Err(e) => exit_with_error!("Could not run clean command: {}", e),
        }
    };

    match status.code() {
        Some(code) => {
            if code != 0 {
                exit_with_error!(
                    "\n{}Cleaning process exited with non-zero exit code: {}{}",
                    Fg(color::Red),
                    code,
                    Fg(color::Reset)
                );
            }
        }
        None => {
            exit_with_error!("Cleaning process stopped unexpectedly");
        }
    };
}
