use termion::color::{Fg, self};

pub fn compile(main_file: Option<String>) {
    // TODO: ask for main file if not provided
    let main_file = main_file.unwrap_or("main.tex".to_string());

    let cmd = format!(
        "latexmk -pdf -bibtex-cond -shell-escape -interaction=nonstopmode {}",
        main_file
    );

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
