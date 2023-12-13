use std::{
    fs::{self, DirEntry, ReadDir},
    io,
    path::{Path, PathBuf},
    process::Command,
};

#[macro_export]
macro_rules! exit_with_error {
    ($($x:expr),+) => {{
        println!($($x),+);
        std::process::exit(1)
    }}
}

pub(crate) use exit_with_error;
use fuzzy_finder::item::Item;

use crate::utils;

pub fn replace_path_placeholders(s: &str, main_file: &Path) -> String {
    let main_file_string = main_file.to_str().unwrap();
    let out = replace_text(s, "<main-file>", main_file_string);

    let stem = main_file_string
        .strip_suffix(".tex")
        .unwrap_or(main_file_string);
    let out = replace_text(out.as_str(), "<main-stem>", stem);

    out
}

fn replace_text(s: &str, pattern: &str, value: &str) -> String {
    let parts = s.split(pattern);
    parts
        .clone()
        .flat_map(|p| [p, value])
        .take(parts.count() * 2 - 1)
        .collect::<String>()
}

/// Clones a repository and returns path to the root of the cloned directory.
pub fn clone_repo(tmp_dir: &Path, url: &str, branch: Option<&String>) -> PathBuf {
    // Path to a temporary directory for cloning repos into.
    let tmp_dir = tmp_dir.join("cloned_repo");

    // Clear the directory: Delete it if it exists and recreate it
    if tmp_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&tmp_dir) {
            exit_with_error!(
                "Could not remove temporary directory '{}': {}",
                tmp_dir.display(),
                e
            );
        }
    }

    utils::create_dir(&tmp_dir);

    // Clone the repo inside the temporary directory
    let status = {
        match match branch {
            Some(b) => Command::new("git")
                .arg("-C")
                .arg(&tmp_dir)
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg("--branch")
                .arg(b)
                .arg(url)
                .status(),
            None => Command::new("git")
                .arg("-C")
                .arg(&tmp_dir)
                .arg("clone")
                .arg(url)
                .status(),
        } {
            Ok(s) => s,
            Err(e) => exit_with_error!("Error running git command: {}", e),
        }
    };

    // Handle git failing
    match status.code() {
        Some(c) if c != 0 => exit_with_error!("Git failed to clone repo. Exit code was {}.", c),
        None => exit_with_error!("Git process stopped unexpectedly"),
        Some(_) => {} // Everything worked!
    }

    // The repo root is the only entry in the temporary directory
    let cloned_repo_root = match read_dir(&tmp_dir).next() {
        Some(Ok(f)) => f.path(),
        Some(Err(e)) => exit_with_error!("Could not open dir '{}': {}", tmp_dir.display(), e),
        None => exit_with_error!("Could not find cloned repository directory."),
    };

    debug_assert!(cloned_repo_root.is_dir());

    cloned_repo_root
}

pub fn read_dir(dir: &Path) -> ReadDir {
    match fs::read_dir(dir) {
        Ok(d) => d,
        Err(e) => exit_with_error!("Could not read directory '{}': {}", dir.display(), e),
    }
}

pub fn create_dir(dir: &Path) {
    if let Err(e) = fs::create_dir(dir) {
        exit_with_error!("Could not create directory '{}': {}", dir.display(), e);
    }
}

pub fn create_dir_all(dir: &Path) {
    if let Err(e) = fs::create_dir_all(dir) {
        exit_with_error!(
            "Could not create directory and its parrents '{}': {}",
            dir.display(),
            e
        );
    }
}

pub fn remove_dir_all(dir: &Path) {
    if let Err(e) = fs::remove_dir_all(dir) {
        exit_with_error!(
            "Could not remove directory with its children '{}': {}",
            dir.display(),
            e
        );
    }
}

pub fn write<C>(file: &Path, contents: C)
where
    C: AsRef<[u8]>,
{
    if let Err(e) = fs::write(file, contents) {
        exit_with_error!("Could not write to file '{}': {}", file.display(), e)
    }
}

pub fn copy(from: &Path, to: &Path) {
    if let Err(e) = fs::copy(from, to) {
        exit_with_error!(
            "Could not copy '{}' to '{}': {}",
            from.display(),
            to.display(),
            e
        )
    }
}

pub fn symlink(from: &Path, to: &Path) {
    if let Err(e) = std::os::unix::fs::symlink(from, to) {
        exit_with_error!(
            "Could not create symling from '{}' to '{}': {}",
            from.display(),
            to.display(),
            e
        );
    };
}

pub fn remove_file(path: &Path) {
    if let Err(e) = fs::remove_file(path) {
        exit_with_error!("Could not remove file '{}': {}", path.display(), e)
    }
}

pub fn handle_file_iter(res: io::Result<DirEntry>) -> DirEntry {
    match res {
        Ok(d) => d,
        Err(e) => exit_with_error!(
            "Some sort of intermittent IO error happened during iteration: {}",
            e
        ),
    }
}

pub fn parrent(path: &Path) -> &Path {
    match path.parent() {
        Some(p) => p,
        None => exit_with_error!("Could not get parrent of '{}'", path.display()),
    }
}

pub fn start_fuzzy_finder<T>(items: Vec<Item<T>>, n: i8) -> Option<T>
where
    T: Clone,
{
    match fuzzy_finder::FuzzyFinder::find(items, n) {
        Ok(res) => res,
        Err(e) => exit_with_error!("Fuzzy finder error: {}", e),
    }
}

pub fn _copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            _copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
