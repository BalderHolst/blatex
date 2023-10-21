use std::{collections::HashMap, fs, path::PathBuf};

use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::config::LOCAL_CONFIG_FILE;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    /// Path to local configuration file
    #[arg(short('C'), long)]
    config_path: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize latex document with a template
    Init {
        /// Name of a template to use
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Compile latex document
    Compile {
        /// Entry point for the latex compiler
        #[clap(index = 1)]
        main_file: Option<String>,
    },

    /// Clean temporary files
    Clean {
        /// Entry point for the latex compiler
        #[clap(index = 1)]
        main_file: Option<String>,
    },

    /// Show errors and warnings from the last compilation
    Log,

    /// Commands for managing templates
    Templates {
        #[clap(subcommand)]
        template_command: TemplateCommand,
    },

    /// Manage blatex configuration
    Config {
        #[clap(subcommand)]
        config_command: ConfigCommand,

        #[arg(short, long, default_value_t = false)]
        global: bool,
    },
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    // Add a local file or directory to templates
    Add {
        /// The path to a zip-file or directory of zip-files
        #[clap(index = 1)]
        path: String,

        /// Symlink instead of copying files to templates directory
        #[arg(long, default_value_t = false)]
        symlink: bool,

        /// Override existing templates
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// Add a git repository to templates
    AddRepo {
        /// The URL to a repository
        #[clap(index = 1)]
        url: String,

        /// The path to the template file or directory within the repository
        #[arg(short, long)]
        path: Option<String>,

        /// Override existing templates
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// List templates
    List,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Create a local or global configuration file with the default options
    Create {
        /// Override existing templates
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// Dump the current configuration to stdout
    Show,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Directory for application data
    pub data_dir: PathBuf,

    /// Directory for storing templates
    pub templates_dir: PathBuf,

    /// Directory used for configuration of blatex
    pub config_file: PathBuf,

    /// Directory used for temporary files
    pub temp_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("com", "blatex", "blatex").unwrap();
        let data_dir = proj_dirs.data_dir().to_path_buf();
        let templates_dir = data_dir.join("templates");
        let config_dir = proj_dirs.config_dir().join("blatex.toml");
        let temp_dir = proj_dirs.cache_dir().join("tmp");

        Config {
            data_dir,
            templates_dir,
            config_file: config_dir,
            temp_dir,
        }
    }
}

impl Config {
    fn override_some_fields(config: &mut Config, map: HashMap<String, toml::Value>) {
        Self::override_if_some_string(&mut config.data_dir, map.get("data_dir"));
        Self::override_if_some_string(&mut config.templates_dir, map.get("templates_dir"));
        Self::override_if_some_string(&mut config.config_file, map.get("config_file"));
        Self::override_if_some_string(&mut config.temp_dir, map.get("temp_dir"));
    }

    fn override_if_some_string(var: &mut PathBuf, value: Option<&toml::Value>) {
        if let Some(toml_value) = value {
            if let toml::Value::String(s) = toml_value {
                *var = PathBuf::from(s);
            }
        }
    }

    pub fn new_global() -> Self {
        let mut config = Config::default();

        if config.config_file.is_file() {
            let global_toml = fs::read_to_string(&config.config_file).unwrap();
            let global_config: HashMap<String, toml::Value> =
                toml::from_str(global_toml.as_str()).unwrap();

            Self::override_some_fields(&mut config, global_config);
        }

        config
    }

    pub fn new_local(local_config_file: Option<PathBuf>) -> Self {
        let mut config = Config::new_global();

        let default_config = local_config_file.is_none();

        let local_config_file = local_config_file.unwrap_or({
            let cwd = std::env::current_dir().unwrap();
            cwd.join(LOCAL_CONFIG_FILE)
        });

        if let Ok(toml) = fs::read_to_string(&local_config_file) {
            let local_config: HashMap<String, toml::Value> = toml::from_str(toml.as_str()).unwrap();
            Self::override_some_fields(&mut config, local_config);
        } else if !default_config {
            eprintln!(
                "Could not read local config file `{}`. Skipping.",
                local_config_file.display()
            );
        }
        config
    }
}

pub struct Opts {
    /// CLI arguments.
    pub args: Args,

    /// Configuration. Options that can be changed by configuration files.
    pub config: Config,
}

impl Opts {
    pub fn create() -> Self {
        let args = Args::parse();

        let config = Config::new_local(args.config_path.clone().map(|s| PathBuf::from(s)));

        Self { args, config }
    }
}
