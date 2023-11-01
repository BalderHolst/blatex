use std::{
    collections::HashMap,
    ffi::OsStr,
    fs, io, os,
    path::{Path, PathBuf},
    process::exit,
};

use termion::{
    color::{self, Fg},
    style,
};

use crate::{
    opts::{Config, RemoteTemplate},
    utils,
};

#[derive(Debug)]
pub enum Template {
    Local(PathBuf),
    Remote(String, RemoteTemplate),
}

impl ToString for Template {
    fn to_string(&self) -> String {
        match self {
            Template::Local(p) => p.to_str().unwrap().to_string(),
            Template::Remote(name, _r) => format!(
                "{}{} (remote){}",
                color::Fg(color::Magenta),
                name,
                color::Fg(color::Reset)
            ),
        }
    }
}

pub fn get_templates<P>(
    templates_dir: P,
    remote_templates: &HashMap<String, RemoteTemplate>,
) -> Vec<Template>
where
    P: AsRef<std::path::Path>,
{
    // Get the local templates
    let mut templates: Vec<Template> = get_local_templates(&templates_dir)
        .iter()
        .map(|t| Template::Local(t.strip_prefix(&templates_dir).unwrap().to_path_buf()))
        .collect();

    // Insert the remote templates
    templates.extend(
        remote_templates
            .iter()
            .map(|(n, t)| Template::Remote(n.clone(), t.clone())),
    );

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
            templates.push(templates_dir.as_ref().join(path.file_name().unwrap()));
        } else if path.is_dir() {
            templates.extend(get_local_templates(path))
        }
    }
    templates
}

/// Search for a template with a name
pub fn search_templates<'a>(
    name: &String,
    templates: &'a Vec<Template>,
) -> Option<&'a Template> {
    let name_path = PathBuf::from(&name);
    for t in templates {
        match t {
            Template::Local(p) => {
                if p == &name_path.with_extension("zip") {
                    return Some(t);
                }
            }
            Template::Remote(n, _r) => {
                if n == name {
                    return Some(t);
                }
            }
        }
    }
    None
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn add_paths(
    cwd: PathBuf,
    config: Config,
    paths: Vec<String>,
    symlink: bool,
    force: bool,
    rename: Option<String>,
) {
    if rename.is_some() && paths.len() != 1 {
        eprintln!("Cannot rename when adding more than one file or directory.");
        exit(1)
    }

    for p in paths {
        add_path(
            &cwd,
            &config,
            PathBuf::from(p),
            symlink,
            force,
            rename.as_ref(),
        );
    }
}

fn add_path(
    cwd: &PathBuf,
    config: &Config,
    path: PathBuf,
    symlink: bool,
    force: bool,
    rename: Option<&String>,
) {
    let path = PathBuf::from(path);
    let path_filename = path.file_name().unwrap();

    if symlink && !cfg!(unix) {
        eprintln!("You can only use symlinks om UNIX systems.");
        exit(1);
    }

    let templates_dir = &config.templates_dir;
    let dest = templates_dir.join(match rename {
        Some(new_name) => {
            let mut p = PathBuf::from(new_name);
            if path.is_file() {
                p.set_extension("zip");
            }
            p
        }
        None => PathBuf::from(path_filename),
    });

    if dest.exists() {
        if !force {
            match path_filename.to_str() {
                Some(n) => eprintln!("Template `{}` already exists. Use --force to override.", n),
                None => eprintln!("Template already exists. Use --force to override."),
            }
            exit(1)
        }
        {
            if dest.is_dir() {
                fs::remove_dir_all(dest.as_path()).unwrap();
            } else {
                fs::remove_file(dest.as_path()).unwrap();
            }
        }
    }

    fs::create_dir_all(templates_dir.as_path()).unwrap();

    // This works for both paths and directories
    if symlink {
        os::unix::fs::symlink(cwd.join(path), dest).unwrap();
        return;
    }

    if path.is_file() {
        if Some(OsStr::new("zip")) != path.extension() {
            eprintln!("Templates should be zip files.");
            exit(1);
        }

        // Make sure that parent of added file exists
        let parrent = dest.parent().unwrap();
        fs::create_dir_all(parrent).unwrap();

        if symlink {
            os::unix::fs::symlink(cwd.join(path), dest).unwrap();
        } else {
            fs::copy(path.as_path(), dest).unwrap();
        }
    } else if path.is_dir() {
        if symlink {
            os::unix::fs::symlink(cwd.join(path), dest).unwrap();
        } else {
            copy_dir_all(path.as_path(), dest).unwrap();
        }
    } else {
        eprintln!("File `{}` is neither file or directory.", path.display());
        exit(1);
    }
}

pub fn list_templates(config: Config) {
    for t in get_templates(&config.templates_dir, &config.remote_templates) {
        println!("{}", t.to_string())
    }
}

fn _list_templates_recursive(dir: PathBuf, level: usize) {
    match fs::read_dir(dir) {
        Ok(read_dir) => {
            for file in read_dir {
                let path = file.unwrap().path();
                if path.is_file() {
                    println!(
                        "{}{}",
                        "  ".repeat(level),
                        path.file_name().unwrap().to_str().unwrap(),
                    );
                } else if path.is_dir() {
                    println!(
                        "{}{}{}{}{}{}",
                        "  ".repeat(level),
                        style::Bold,
                        Fg(color::Blue),
                        path.file_name().unwrap().to_str().unwrap(),
                        Fg(color::Reset),
                        style::Reset,
                    );
                    _list_templates_recursive(path, level + 1)
                }
            }
        }
        Err(_) => {
            println!("No templates found.")
        }
    }
}

pub fn add_repo(cwd: PathBuf, config: Config, url: String, path: Option<String>, force: bool) {
    // TODO: support branches
    let cloned_repo_root = utils::clone_repo(&config.temp_dir, url.as_str(), None);

    // Handle that the user may provide a path within repo as the template
    let template_path = match path {
        Some(sub_path) => {
            let p = cloned_repo_root.join(&sub_path);
            if !p.is_dir() {
                eprintln!(
                    "Path `{}` is not a directory within repository at `{}`.",
                    sub_path, url
                );
                exit(1);
            }
            p
        }
        None => cloned_repo_root,
    };

    // The zip archive will have the same name as the repo, but with the .zip extension
    let archive_path = config
        .temp_dir
        .join(template_path.file_name().unwrap())
        .with_extension("zip");

    // Create the zip archive
    zip_extensions::write::zip_create_from_directory(&archive_path, &template_path).unwrap();

    // Add the template as a normal local template
    add_path(&cwd, &config, archive_path, false, force, None)
}
