use std::path::PathBuf;

use termion::color::{self, Fg};

use crate::{
    exit_with_error, log,
    opts::{CompileArgs, Config},
    utils,
};

pub fn compile(cwd: PathBuf, config: Config, args: CompileArgs) {
    let main_file = match args.main_file {
        Some(f) => PathBuf::from(f),
        None => config.main_file.clone(),
    };
    compile_file(cwd, config, main_file);
}

pub fn compile_file(cwd: PathBuf, config: Config, main_file: PathBuf) {
    let cmd = utils::replace_text(
        &config.compile_cmd,
        "<main-file>",
        main_file.display().to_string().as_str(),
    );
    let prefix = format!("cd \"{}\"", config.root.display());

    let cmd = prefix + " && " + cmd.as_str();

    println!(
        "{}Running command: `{}`{}\n",
        Fg(color::Blue),
        cmd,
        Fg(color::Reset)
    );

    let status = if cfg!(target_os = "windows") {
        exit_with_error!("Compilation on windows is currently not supported.");
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
                exit_with_error!(
                    "\n{}Compilation process exited with non-zero exit code: {}{}",
                    Fg(color::Red),
                    code,
                    Fg(color::Reset)
                );
            }
        }
        None => {
            exit_with_error!("Compilation process stopped unexpectedly");
        }
    };

    // Parse log file
    log::print_log(cwd, &main_file);
}
