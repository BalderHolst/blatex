use std::{fs, io::Cursor, path::PathBuf, collections::HashMap};

use fuzzy_finder::item::Item;

use crate::{
    config::{self, LOCAL_CONFIG_FILE},
    opts::{Config, RemoteTemplate},
};

enum Template {
    Local(PathBuf),
    Remote(String, RemoteTemplate),
}

fn get_templates<P>(templates_dir: P, remote_templates: HashMap<String, RemoteTemplate>) -> Vec<Template>
where
    P: AsRef<std::path::Path>,
{
    // Get the local templates
    let mut templates: Vec<Template> = get_local_templates(templates_dir)
        .iter()
        .map(|t| Template::Local(t.clone()))
        .collect();

    // Insert the remote templates
    templates.extend(remote_templates.iter().map(|(n, t)| Template::Remote(n.clone() + " (remote)", t.clone())));

    return templates;
}

fn get_local_templates<P>(templates_dir: P) -> Vec<PathBuf>
where
    P: AsRef<std::path::Path>,
{
    let mut templates = Vec::new();
    for file in fs::read_dir(&templates_dir).unwrap() {
        let path = file.unwrap().path();
        if path.is_file() {
            templates.push(templates_dir.as_ref().join(path.file_name().unwrap()))
        } else if path.is_dir() {
            templates.extend(get_local_templates(path))
        }
    }
    templates
}

pub fn init(cwd: PathBuf, config: Config, template: Option<String>) {
    let templates_dir = &config.templates_dir;
    let templates = get_templates(templates_dir.as_path(), config.remote_templates);

    let template_path = match template {
        Some(p) => {
            // TODO: Handle remote templates
            templates_dir.join(p).with_extension("zip")
        },
        None => {
            // Create fuzzy finder items
            let items: Vec<Item<&Template>> = templates
                .iter()
                .map(|t| {
                    match t {
                        Template::Local(p) => {
                            Item::new(
                                p.strip_prefix(templates_dir.as_path())
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string(),
                                t,
                                )
                        },
                        Template::Remote(name, _r) => Item::new(name.clone(), t),
                    }
                })
                .collect();

            // Calculate number of items depending on height of the terminal window
            let nr_of_items = match termion::terminal_size() {
                Ok((_cols, rows)) => u16::min(8, rows / 5 * 3),
                Err(_) => 8,
            };

            // Run the fuzzy finder
            match fuzzy_finder::FuzzyFinder::find(items, nr_of_items as i8).unwrap() {
                Some(Template::Local(p)) => p.to_path_buf(),
                Some(Template::Remote(name, remote)) => todo!(),
                None => {
                    eprintln!("No template chosen.");
                    std::process::exit(1);
                }
            }
        }
    };

    dbg!(&template_path);
    let archive_bytes = fs::read(template_path).unwrap();

    zip_extract::extract(Cursor::new(archive_bytes), &cwd, true).unwrap();

    // Create configuration file if it does not exist
    let config_file_path = PathBuf::from(LOCAL_CONFIG_FILE);
    if !config_file_path.exists() {
        config::create(&cwd, false, false);
    }

    // Create new configuration
    let config = Config::new_local(&cwd, Some(config_file_path));

    // Compile document with the new configuration
    let main_file = config.main_file.clone();
    crate::compile::compile(cwd, config, &main_file);
}
