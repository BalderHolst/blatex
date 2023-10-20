use std::{
    ffi::OsStr,
    fs, io, os,
    path::{Path, PathBuf},
    process::exit,
};

use termion::{color::{self, Fg}, style};

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
pub fn add_path(path: String, symlink: bool, templates_dir: PathBuf, force: bool) {
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
