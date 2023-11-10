use std::{fs, path::Path};

use crate::{
    exit_with_error,
    opts::{Config, ConfigCreateArgs},
};

pub const LOCAL_CONFIG_FILE: &str = ".blatex.toml";

pub fn create(cwd: &Path, global: bool, args: &ConfigCreateArgs, config: &Config) {
    let description = if global {
        r#"# This file is the template used when creating local configuration files.
# Options here will always get read, but may be overridden by local
# config files. If you delete options, they will simply be set to their
# default value.
"#
    } else {
        "# This is your local configuration for this project.\n# Options here will override global ones.\n"
    };

    let config_string = match toml::to_string_pretty(&config) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("WARNING: Could not convert configuration to toml.");
            "".to_string()
        }
    };

    let toml = format!("{}{}", description, config_string);

    let dest = if global {
        Config::default().config_file
    } else {
        cwd.join(LOCAL_CONFIG_FILE)
    };

    if !args.force && dest.exists() {
        exit_with_error!(
            "File `{}` already exists. Run with --force to override.",
            dest.display()
        );
    }

    // Create directory if it does not exist
    fs::create_dir_all(dest.parent().unwrap()).unwrap();

    fs::write(&dest, toml).unwrap();
    println!(
        "Wrote {} config `{}`",
        if global { "global" } else { "local" },
        dest.display()
    );
}

pub fn show(config: Config, global: bool) {
    let config = if global { Config::new_global() } else { config };
    println!(
        "{}",
        toml::to_string_pretty(&config).unwrap_or("".to_string())
    );
}
