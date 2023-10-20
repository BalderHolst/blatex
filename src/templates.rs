use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io, os,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use termion::{
    color::{self, Fg},
    style,
};

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

// TODO: Glob support
// TODO: Rename support
pub fn add_path(path: PathBuf, symlink: bool, templates_dir: PathBuf, force: bool) {
    let path = PathBuf::from(path);
    let path_filename = path.file_name().unwrap();

    if symlink && !cfg!(unix) {
        eprintln!("You can only use symlinks om UNIX systems.");
        exit(1);
    }

    let dest = templates_dir.join(path_filename);

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

    // This works for both paths and directories
    if symlink {
        os::unix::fs::symlink(std::env::current_dir().unwrap().join(path), dest).unwrap();
        return;
    }

    if path.is_file() {
        if Some(OsStr::new("zip")) != path.extension() {
            eprintln!("Templates should be zip files.");
            exit(1);
        }
        fs::create_dir_all(templates_dir.as_path()).unwrap();

        if symlink {
            os::unix::fs::symlink(std::env::current_dir().unwrap().join(path), dest).unwrap();
        } else {
            fs::copy(path.as_path(), dest).unwrap();
        }
    } else if path.is_dir() {
        let dest = templates_dir.join(path.file_name().unwrap());
        if symlink {
            os::unix::fs::symlink(std::env::current_dir().unwrap().join(path), dest).unwrap();
        } else {
            copy_dir_all(path.as_path(), dest).unwrap();
        }
    } else {
        eprintln!("File `{}` is neither file or directory.", path.display());
        exit(1);
    }
}

pub fn list_templates(templates_dir: PathBuf) {
    list_templates_recursive(templates_dir, 0)
}

fn list_templates_recursive(dir: PathBuf, level: usize) {
    for file in fs::read_dir(dir).unwrap() {
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
            list_templates_recursive(path, level + 1)
        }
    }
}

pub fn add_repo(
    url: String,
    path: Option<String>,
    templates_dir: PathBuf,
    force: bool,
    tmp_dir: PathBuf,
) {

    // Path to a temporary directory for cloning repos into.
    let tmp_dir = tmp_dir.join("cloned_repo");

    // Clear the directory: Delete it if it exists and recreate it
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).unwrap();
    }
    fs::create_dir(&tmp_dir).unwrap();

    // Clone the repo inside the temporary directory
    let status = Command::new("git")
        .arg("-C")
        .arg(&tmp_dir)
        .arg("clone")
        .arg(&url)
        .status()
        .unwrap();

    // Handle git failing
    match status.code() {
        Some(c) => {
            if c != 0 {
                eprintln!("Git failed to clone repo. Exit code was {}.", c);
                exit(1);
            }
        }
        None => {
            eprintln!("Git process stopped unexpectedly");
            exit(1);
        }
    }

    // The repo root is the only entry in the temporary directory
    let cloned_repo_root = fs::read_dir(&tmp_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();

    debug_assert!(cloned_repo_root.is_dir());

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
    let archive_path = tmp_dir.join(template_path.file_name().unwrap()).with_extension("zip");

    // Create the zip archive
    zip_extensions::write::zip_create_from_directory(&archive_path, &template_path).unwrap();

    // Add the template as a normal local template
    add_path(archive_path, false, templates_dir, force)
}

