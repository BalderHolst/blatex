use std::{
    fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

pub fn replace_text(s: &str, pattern: &str, value: &str) -> String {
    let (first, second) = s.split_once(pattern).expect("pattern not found.");
    first.to_string() + value + second
}

/// Clones a repository and returns path to the root of the cloned directory.
pub fn clone_repo(tmp_dir: &Path, url: &str, branch: Option<&String>) -> PathBuf {
    // Path to a temporary directory for cloning repos into.
    let tmp_dir = tmp_dir.join("cloned_repo");

    // Clear the directory: Delete it if it exists and recreate it
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).unwrap();
    }
    fs::create_dir(&tmp_dir).unwrap();

    // Clone the repo inside the temporary directory
    let status = {
        match branch {
            Some(b) => Command::new("git")
                .arg("-C")
                .arg(&tmp_dir)
                .arg("clone")
                .arg("--branch")
                .arg(b)
                .arg(url)
                .status()
                .unwrap(),
            None => Command::new("git")
                .arg("-C")
                .arg(&tmp_dir)
                .arg("clone")
                .arg(url)
                .status()
                .unwrap(),
        }
    };

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

    cloned_repo_root
}
