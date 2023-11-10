use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use fuzzy_finder::item::Item;
use termion::color;
use zip_extensions::zip_extract;

use crate::{
    config::{self, LOCAL_CONFIG_FILE},
    exit_with_error,
    opts::{Config, ConfigCreateArgs, InitArgs, RemoteTemplate},
    templates::{self, Template},
    utils,
};

fn clone_remote_template(tmp_dir: &Path, name: &String, remote: &RemoteTemplate) -> PathBuf {
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

fn copy_directory(src: &Path, dest: &Path) {
    utils::create_dir(dest);
    for file in utils::read_dir(src) {
        let file = utils::handle_file_iter(file).path();
        let file_name = file.file_name().unwrap_or(OsStr::new("no-file-name"));
        if file.is_file() {
            utils::copy(&file, dest.join(file_name).as_path());
        } else if file.is_dir() {
            copy_directory(&src.join(file_name), &dest.join(file_name))
        }
    }
}

pub fn init(cwd: PathBuf, mut config: Config, args: InitArgs) {
    // Make sure that the folder is not already initialized
    if config.root.join(LOCAL_CONFIG_FILE).exists() {
        println!("Document already initialized.");
        exit(0);
    }

    let c = utils::read_dir(&config.root).count();
    if c == 0 {
        let templates_dir = &config.templates_dir;
        let templates = templates::get_templates(templates_dir.as_path(), &config.remote_templates);

        let template_path = match args.template {
            Some(t) => match templates::search_templates(&t, &templates) {
                Some(Template::Local(p)) => config.templates_dir.join(p),
                Some(Template::Remote { name, remote }) => {
                    config = remote.config.clone();
                    clone_remote_template(&config.temp_dir, name, remote)
                }
                None => exit_with_error!("Could not find template '{}'.", t),
            },
            None => {
                // Create fuzzy finder items
                let items: Vec<Item<&Template>> = templates
                    .iter()
                    .map(|t| Item::new(t.to_string(), t))
                    .collect();

                // Calculate number of items depending on height of the terminal window and number of
                // templates.
                let nr_of_items = match termion::terminal_size() {
                    Ok((_cols, rows)) => u16::min(items.len() as u16, rows / 5 * 3),
                    Err(_) => 8,
                };

                // Run the fuzzy finder
                match utils::start_fuzzy_finder(items, nr_of_items as i8) {
                    Some(Template::Local(p)) => config.templates_dir.join(p),
                    Some(Template::Remote { name, remote }) => {
                        config = remote.config.clone();
                        clone_remote_template(&config.temp_dir, name, remote)
                    }
                    None => exit_with_error!("No template chosen."),
                }
            }
        };

        // If the template is an archive, extract it to the current working directory
        if template_path.is_file() {
            if let Err(e) = zip_extract(&template_path, &cwd) {
                exit_with_error!(
                    "Could not extract zip archive '{}' to '{}': {}",
                    template_path.display(),
                    cwd.display(),
                    e
                )
            }
        }
        // If template path is a directory (can happen when using remote templates), simply copy its
        // contents.
        else {
            for file in utils::read_dir(&template_path) {
                let file = utils::handle_file_iter(file).path();
                let file_name = file.file_name().unwrap_or(OsStr::new("no-file-name"));
                let dest = cwd.join(file_name);
                if file.is_dir() {
                    copy_directory(&file, &dest)
                } else {
                    utils::copy(&file, &dest);
                }
            }
        }
    }

    // Create configuration file if it does not exist
    let config_file_path = PathBuf::from(LOCAL_CONFIG_FILE);
    if !config_file_path.exists() {
        // if the main file in the config does not exist, force the user to pick an existing one
        // with a fuzzy finder.
        if !config.root.join(&config.main_file).exists() {
            if let Ok(dir) = fs::read_dir(&config.root) {
                let items: Vec<fuzzy_finder::item::Item<PathBuf>> = dir
                    .filter_map(|file| {
                        let file = utils::handle_file_iter(file);
                        if file.path().is_file() {
                            let file_name = file.file_name().to_str().map(|s| s.to_string());
                            file_name
                                .map(|s| fuzzy_finder::item::Item::new(s.clone(), PathBuf::from(s)))
                        } else {
                            None
                        }
                    })
                    .collect();
                println!(
                    "Could not find main file '{}' please choose one.",
                    config.main_file.display()
                );
                let l = items.len();
                match fuzzy_finder::FuzzyFinder::find(items, i8::min(l as i8, 8)) {
                    Ok(Some(p)) => config.main_file = p,
                    _ => exit_with_error!("\nNo file chosen."),
                }
            }
        }
        config::create(&cwd, false, &ConfigCreateArgs { force: false }, &config);
        println!()
    }

    // Compile document with the new configuration
    let main_file = config.main_file.clone();
    crate::compile::compile_file(config, main_file);
}
