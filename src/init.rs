use std::{fs, io::Cursor, path::PathBuf, process::exit};

use fuzzy_finder::item::Item;
use termion::color;

use crate::{
    config::{self, LOCAL_CONFIG_FILE},
    opts::{Config, RemoteTemplate, InitArgs, ConfigCreateArgs},
    templates::{self, Template},
    utils,
};

fn clone_remote_template(tmp_dir: &PathBuf, name: &String, remote: &RemoteTemplate) -> PathBuf {
    println!(
        "\n{}Cloning template '{}' from '{}'.{}",
        color::Fg(color::Blue),
        name,
        &remote.url,
        color::Fg(color::Reset)
    );
    let template_root = utils::clone_repo(tmp_dir, remote.url.as_str(), remote.branch.as_ref());
    if let Some(path) = &remote.path {
        template_root.join(path)
    } else {
        template_root
    }
}

fn copy_directory(src: &PathBuf, dest: &PathBuf) {
    fs::create_dir(&dest).unwrap();
    for file in fs::read_dir(src).unwrap() {
        let file = file.unwrap().path();
        let file_name = file.file_name().unwrap();
        if file.is_file() {
            fs::copy(&file, dest.join(file_name)).unwrap();
        } else if file.is_dir() {
            copy_directory(&src.join(file_name), &dest.join(file_name))
        }
    }
}

pub fn init(cwd: PathBuf, config: Config, args: InitArgs) {
    let templates_dir = &config.templates_dir;
    let templates = templates::get_templates(templates_dir.as_path(), &config.remote_templates);

    let template_path = match args.template {
        Some(t) => match templates::search_templates(&t, &templates) {
            Some(Template::Local(p)) => config.templates_dir.join(p),
            Some(Template::Remote(name, remote)) => {
                clone_remote_template(&config.temp_dir, name, remote)
            }
            None => {
                eprintln!("Could not find template '{}'.", t);
                exit(1)
            }
        },
        None => {
            // Create fuzzy finder items
            let items: Vec<Item<&Template>> = templates
                .iter()
                .map(|t| Item::new(t.to_string(), t))
                .collect();

            // Calculate number of items depending on height of the terminal window
            let nr_of_items = match termion::terminal_size() {
                Ok((_cols, rows)) => u16::min(items.len() as u16, rows / 5 * 3),
                Err(_) => 8,
            };

            // Run the fuzzy finder
            match fuzzy_finder::FuzzyFinder::find(items, nr_of_items as i8).unwrap() {
                Some(Template::Local(p)) => p.to_path_buf(),
                Some(Template::Remote(name, remote)) => {
                    clone_remote_template(&config.temp_dir, name, remote)
                }
                None => {
                    eprintln!("No template chosen.");
                    exit(1);
                }
            }
        }
    };

    // If the template is an archive, extract it to the current working directory
    if template_path.is_file() {
        let archive_bytes = fs::read(template_path).unwrap();
        zip_extract::extract(Cursor::new(archive_bytes), &cwd, true).unwrap();
    }
    // If template path is a directory (can happen when using remote templates), simply copy its
    // contents.
    else {
        for file in fs::read_dir(template_path).unwrap() {
            let file = file.unwrap().path();
            let file_name = file.file_name().unwrap().to_str().unwrap();
            let dest = cwd.join(file_name);
            if file.is_dir() {
                copy_directory(&file, &dest)
            } else {
                fs::copy(&file, dest).unwrap();
            }
        }
    }

    // Create configuration file if it does not exist
    let config_file_path = PathBuf::from(LOCAL_CONFIG_FILE);
    if !config_file_path.exists() {
        config::create(&cwd, false, &ConfigCreateArgs {
            force: false,
        });
    }

    // Create new configuration
    let config = Config::new_local(&cwd, Some(config_file_path));

    // Compile document with the new configuration
    let main_file = config.main_file.clone();
    crate::compile::compile_file(cwd, config, &main_file);
}
