mod cli;
mod templates;

use std::{fs, io::Cursor, path::PathBuf};

use cli::{Command, Opts};
use fuzzy_finder::{self, item::Item};
use termion::color::{self, Fg};

fn compile(main_file: Option<String>) {
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

fn clean(main_file: Option<String>) {
    // TODO: ask for main file if not provided
    let main_file = main_file.unwrap_or("main.tex".to_string());
    let cmd = format!("latexmk -c {}", main_file);

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

fn get_templates<P>(dir: P) -> Vec<PathBuf>
where
    P: AsRef<std::path::Path>,
{
    let mut templates = Vec::new();
    for file in fs::read_dir(&dir).unwrap() {
        let path = file.unwrap().path();
        if path.is_file() {
            templates.push(dir.as_ref().join(path.file_name().unwrap()))
        } else if path.is_dir() {
            templates.extend(get_templates(path))
        }
    }

    templates
}

fn init(templates_dir: PathBuf, template: Option<String>) {
    let templates = get_templates(templates_dir.as_path());

    let template_path = match template {
        Some(p) => templates_dir.join(p),
        None => {

            // Create fuzzy finder items
            let items: Vec<Item<&PathBuf>> = templates
                .iter()
                .map(|p| {
                    Item::new(
                        p.strip_prefix(templates_dir.as_path())
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                        p,
                        )
                })
            .collect();

            // Calculate number of items depending on height of the terminal window
            let nr_of_items = match termion::terminal_size() {
                Ok((_cols, rows)) => u16::min(8, rows / 5 * 3),
                Err(_) => 8,
            };

            // Run the fuzzy finder
            match fuzzy_finder::FuzzyFinder::find(items, nr_of_items as i8).unwrap() {
                Some(p) => p,
                None => {
                    eprintln!("No template chosen.");
                    std::process::exit(1);
                }
            }.to_path_buf()
        }
    };

    let cwd = std::env::current_dir().unwrap();

    let archive_bytes = fs::read(template_path).unwrap();

    zip_extract::extract(Cursor::new(archive_bytes), &cwd, true).unwrap();

    compile(None)
}

fn main() {
    let opts = Opts::create();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match opts.args.command {
        Command::Init { template } => init(opts.templates_dir, template),
        Command::Compile { main_file } => compile(main_file),
        Command::Clean { main_file } => clean(main_file),
        Command::Log => todo!(),
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
