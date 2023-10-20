use std::{path::PathBuf, fs, io::Cursor};

use fuzzy_finder::item::Item;

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

pub fn init(templates_dir: PathBuf, template: Option<String>) {
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
            }
            .to_path_buf()
        }
    };

    let cwd = std::env::current_dir().unwrap();

    let archive_bytes = fs::read(template_path).unwrap();

    zip_extract::extract(Cursor::new(archive_bytes), &cwd, true).unwrap();

    crate::compile::compile(None)
}