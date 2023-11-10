use std::path::Path;

use crate::{
    exit_with_error,
    opts::{Config, ConfigCreateArgs},
    utils,
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
    let config_dir = match dest.parent() {
        Some(d) => d,
        None => exit_with_error!("Cannot find parrent directory for '{}'.", dest.display()),
    };
    utils::create_dir_all(config_dir);

    utils::write(&dest, toml);
    println!(
        "Wrote {} config `{}`",
        if global { "global" } else { "local" },
        dest.display()
    );
}

pub fn show(config: Config, global: bool) {
    let config = if global { Config::new_global() } else { config };
    let config_string = match toml::to_string_pretty(&config) {
        Ok(s) => s,
        Err(e) => exit_with_error!("Could not convert configuration to string: {}", e),
    };
    println!("{}", config_string);
}
