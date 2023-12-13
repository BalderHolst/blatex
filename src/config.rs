use std::path::Path;

use crate::{
    exit_with_error,
    opts::{Config, ConfigCreateArgs},
    utils,
};

pub const LOCAL_CONFIG_FILE: &str = ".blatex.toml";

pub fn create(cwd: &Path, global: bool, args: &ConfigCreateArgs, config: &Config) {
    let toml = match global {
        true => create_global_configuration_string(config),
        false => create_local_configuration_string(config),
    };

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

fn create_local_configuration_string(config: &Config) -> String {
    let desc = "# This is your local configuration for this document.\n# Options here will override global ones.\n";

    let config_string = format!(
        r#"
main_file = "{main_file}"
compile_cmd = "{compile_cmd}"
clean_cmd = "{clean_cmd}"
                                "#,
        main_file = config.main_file.display(),
        compile_cmd = config.compile_cmd,
        clean_cmd = config.clean_cmd
    );

    format!("{}{}", desc, config_string)
}

fn create_global_configuration_string(config: &Config) -> String {
    let desc = r#"
# This is your global configuration file. Options here will always get
# read, but may be overridden by local config files. If you delete
# options, they will simply be set to their default value.
"#;

    let config_string = match toml::to_string_pretty(&config) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("WARNING: Could not convert configuration to toml.");
            "".to_string()
        }
    };

    format!("{}{}", desc, config_string)
}

pub fn show(config: Config, global: bool) {
    let config = if global { Config::new_global() } else { config };
    let config_string = match toml::to_string_pretty(&config) {
        Ok(s) => s,
        Err(e) => exit_with_error!("Could not convert configuration to string: {}", e),
    };
    println!("{}", config_string);
}
