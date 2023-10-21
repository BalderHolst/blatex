use std::{fs, process::exit};

use crate::opts::Config;

pub const LOCAL_CONFIG_FILE: &str = ".blatex.toml";

pub fn create(global: bool, force: bool) {
    let config = Config::default();

    let description = if global {
        r#"# This file is the template used when creating local configuration files.
# Options here will always get read, but may be overridden by local
# config files. If you delete options, they will simply be set to their
# default value.
"#
    } else {
        "# This is your local configuration for this project.\n# Options here will override global ones.\n"
    };

    let toml = format!(
        "{}{}",
        description,
        toml::to_string_pretty(&config).unwrap()
    );

    let dest = if global {
        Config::default().config_file
    } else {
        std::env::current_dir().unwrap().join(LOCAL_CONFIG_FILE)
    };

    if !force && dest.exists() {
        eprintln!(
            "File `{}` already exists. Run with --force to override.",
            dest.display()
        );
        exit(1)
    }

    // Create directory if it does not exist
    fs::create_dir_all(dest.parent().unwrap()).unwrap();

    fs::write(&dest, toml).unwrap();
    println!(
        "Wrote {} default config `{}`",
        if global { "global" } else { "local" },
        dest.display()
    );
}

pub fn show(config: Config, global: bool) {
    let config = if global { Config::new_global() } else { config };
    println!("{}", toml::to_string_pretty(&config).unwrap());
}
