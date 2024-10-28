use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use termion::{
    color::{self, Fg},
    style,
};

use crate::{
    exit_with_error,
    opts::{Config, RemoteTemplate, TemplateAddArgs, TemplateAddRepoArgs},
    utils,
};

#[derive(Debug)]
pub enum Template {
    Local(PathBuf),
    Remote {
        name: String,
        remote: Box<RemoteTemplate>,
    },
}

impl ToString for Template {
    fn to_string(&self) -> String {
        match self {
            Template::Local(p) => p.to_str().unwrap_or("invalid-file-name").to_string(),
            Template::Remote { name, remote: _ } => format!(
                "{}{} (remote){}",
                color::Fg(color::Magenta),
                name,
                color::Fg(color::Reset)
            ),
        }
    }
}

pub fn get_templates(config: &Config) -> Vec<Template> {
    // Get the local templates
    let mut templates: Vec<Template> = get_local_templates(&config.templates_dir)
        .iter()
        .map(|t| {
            Template::Local(
                t.strip_prefix(&config.templates_dir)
                    .expect("local templates should always be in the template directory")
                    .to_path_buf(),
            )
        })
        .collect();

    // Insert the remote templates
    templates.extend(
        config
            .remote_templates
            .iter()
            .map(|(n, t)| Template::Remote {
                name: n.clone(),
                remote: Box::new(t.clone()),
            }),
    );

    templates
}

fn get_local_templates<P>(templates_dir: P) -> Vec<PathBuf>
where
    P: AsRef<std::path::Path>,
{
    let mut templates = Vec::new();

    if !templates_dir.as_ref().exists() {
        return templates;
    }

    for file in utils::read_dir(templates_dir.as_ref()) {
        let path = utils::handle_file_iter(file).path();
        if path.is_file() {
            templates.push(
                templates_dir
                    .as_ref()
                    .join(path.file_name().expect("This should always be a file.")),
            );
        } else if path.is_dir() {
            templates.extend(get_local_templates(path))
        }
    }
    templates
}

/// Search for a template with a name
pub fn search_templates<'a>(name: &String, templates: &'a Vec<Template>) -> Option<&'a Template> {
    let name_path = PathBuf::from(&name);
    for t in templates {
        match t {
            Template::Local(p) => {
                if p == &name_path.with_extension("zip") {
                    return Some(t);
                }
            }
            Template::Remote { name: n, remote: _ } => {
                if n == name {
                    return Some(t);
                }
            }
        }
    }
    None
}

pub fn add_paths(cwd: PathBuf, config: Config, args: TemplateAddArgs) {
    if args.rename.is_some() && args.paths.len() != 1 {
        exit_with_error!("Cannot rename when adding more than one file or directory.");
    }

    for p in args.paths {
        add_path(
            &cwd,
            &config,
            PathBuf::from(p),
            args.symlink,
            args.force,
            args.rename.as_ref(),
        );
    }
}

fn add_path(
    cwd: &Path,
    config: &Config,
    mut path: PathBuf,
    symlink: bool,
    force: bool,
    rename: Option<&String>,
) {
    if path.is_relative() {
        path = cwd.join(path);
    }

    let path_filename = match path.file_name() {
        Some(n) => n,
        None => exit_with_error!("Cannot find file name for path '{}'.", path.display()),
    };

    if symlink && !cfg!(unix) {
        exit_with_error!("You can only use symlinks om UNIX systems.");
    }

    let templates_dir = &config.templates_dir;
    let dest = templates_dir
        .join(match rename {
            Some(new_name) => PathBuf::from(new_name),
            None => PathBuf::from(path_filename),
        })
        .with_extension("zip");

    if dest.exists() {
        if !force {
            let e = match path_filename.to_str() {
                Some(n) => format!("Template `{}` already exists. Use --force to override.", n),
                None => "Template already exists. Use --force to override.".to_string(),
            };
            exit_with_error!("{e}");
        }
        if dest.is_dir() {
            utils::remove_dir_all(dest.as_path());
        } else {
            utils::remove_file(dest.as_path());
        }
    }

    utils::create_dir_all(templates_dir.as_path());

    // This works for both paths and directories
    if symlink {
        let src = cwd.join(path);
        utils::symlink(&src, &dest);
        return;
    }

    if path.is_dir() {
        let tmp_archive_path = config.temp_dir.join(path_filename).with_extension("zip");
        if let Err(e) = zip_extensions::write::zip_create_from_directory(&tmp_archive_path, &path) {
            exit_with_error!(
                "Could not create zip archive from directory '{}': {}",
                path.display(),
                e
            );
        }
        path = tmp_archive_path;
    }

    if !path.is_file() {
        exit_with_error!("File `{}` is neither a file or directory.", path.display());
    }

    if Some(OsStr::new("zip")) != path.extension() {
        exit_with_error!("Templates should be zip files or directories.");
    }

    // Make sure that parent of added file exists
    let parrent = utils::parrent(&dest);
    utils::create_dir_all(parrent);

    if symlink {
        let src = cwd.join(path);
        utils::symlink(&src, &dest)
    } else {
        utils::copy(&path, &dest);
    }
}

pub fn list_templates(config: Config) {
    for t in get_templates(&config) {
        println!("{}", t.to_string())
    }
}

fn _list_templates_recursive(dir: PathBuf, level: usize) {
    match fs::read_dir(dir) {
        Ok(read_dir) => {
            for file in read_dir {
                let path = utils::handle_file_iter(file).path();
                if path.is_file() {
                    println!(
                        "{}{}",
                        "  ".repeat(level),
                        path.file_name()
                            .expect("files shoud not be able to cause errors")
                            .to_str()
                            .unwrap_or("invalid-file-name"),
                    );
                } else if path.is_dir() {
                    println!(
                        "{}{}{}{}{}{}",
                        "  ".repeat(level),
                        style::Bold,
                        Fg(color::Blue),
                        path.file_name()
                            .expect("files shoud not be able to cause errors")
                            .to_str()
                            .unwrap_or("invalid-file-name"),
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

pub fn add_repo(cwd: PathBuf, config: Config, args: TemplateAddRepoArgs) {
    let cloned_repo_root =
        utils::clone_repo(&config.temp_dir, args.url.as_str(), args.branch.as_ref());

    let mut is_zip = false;

    // Handle that the user may provide a path within repo as the template
    let template_path = match args.path {
        Some(sub_path) => {
            let p = cloned_repo_root.join(&sub_path);

            is_zip = p.extension() == Some(OsStr::new("zip"));

            // Check that path is valid
            if !p.is_dir() && !is_zip {
                exit_with_error!(
                    "Path `{}` is not a directory, or zip archive, within repository at `{}`.",
                    sub_path,
                    args.url
                );
            }
            p
        }
        None => cloned_repo_root,
    };

    let template_file_name = match template_path.file_name() {
        Some(n) => match &args.branch {
            Some(branch) => OsString::from_iter([OsStr::new(branch), OsStr::new("@"), n]),
            None => n.to_os_string(),
        },
        None => exit_with_error!(
            "Could not determine template file name from path '{}'.",
            template_path.display()
        ),
    };

    // The zip archive will have the same name as the repo, but with the .zip extension
    let archive_path = match is_zip {
        true => template_path,
        false => {
            let archive_path = config
                .temp_dir
                .join(template_file_name)
                .with_extension("zip");
            if let Err(e) =
                zip_extensions::write::zip_create_from_directory(&archive_path, &template_path)
            {
                exit_with_error!(
                    "Could not create zip archive from directory '{}': {}",
                    archive_path.display(),
                    e
                );
            }
            archive_path
        }
    };

    // Add the template as a normal local template
    add_path(
        &cwd,
        &config,
        archive_path,
        false,
        args.force,
        args.rename.as_ref(),
    )
}
