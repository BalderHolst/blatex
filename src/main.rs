mod cli;
mod templates;

use cli::{Command, Opts};
use termion::color::{Fg, self};

fn compile(main_file: Option<String>) {
    let main_file = main_file.unwrap_or("main.tex".to_string());
    let cmd = format!("latexmk -pdf -bibtex-cond -shell-escape -interaction=nonstopmode {}", main_file);

    println!("{}Running command: `{}`{}\n", Fg(color::Blue), cmd, Fg(color::Reset));

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
        Some(code) => if code != 0 {
            eprintln!("\n{}Compilation process exited with non-zero exit code: {}{}", Fg(color::Red), code, Fg(color::Reset));
            std::process::exit(1)
        },
        None => {
            eprintln!("Compilation process stopped unexpectedly");
            std::process::exit(1)
        },
    };
}

fn clean(main_file: Option<String>) {
    let main_file = main_file.unwrap_or("main.tex".to_string());
    let cmd = format!("latexmk -c {}", main_file);

    println!("{}Running command: `{}`{}\n", Fg(color::Blue), cmd, Fg(color::Reset));

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
        Some(code) => if code != 0 {
            eprintln!("\n{}Cleaning process exited with non-zero exit code: {}{}", Fg(color::Red), code, Fg(color::Reset));
            std::process::exit(1)
        },
        None => {
            eprintln!("Cleaning process stopped unexpectedly");
            std::process::exit(1)
        },
    };
}

fn main() {
    let opts = Opts::create();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match opts.args.command {
        Command::Init => todo!(),
        Command::Compile { file } => compile(file),
        Command::Clean { file } => clean(file),
        Command::Errors => todo!(),
        Command::Templates { template_command } => match template_command {
            cli::TemplateCommand::Add {
                path,
                symlink,
                force,
            } => templates::add_path(path, symlink, opts.templates_dir, force),
            cli::TemplateCommand::AddRepo { url, path } => todo!(),
            cli::TemplateCommand::List => templates::list_templates(opts.templates_dir),
        },
    }
}
