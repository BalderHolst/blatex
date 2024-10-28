use std::{collections::HashMap, fs, path::PathBuf};

use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use toml::map::Map;

use crate::{config::LOCAL_CONFIG_FILE, exit_with_error};

pub const REMOTE_TEMPLATES_OPTION: &str = "remote_templates";

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Clone)]
#[command(name = "blatex")]
#[command(author = "Balder W. Holst <balderwh@gmail.com>")]
#[command(version = VERSION)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    /// Path to local configuration file
    #[arg(short('C'), long)]
    config_path: Option<String>,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Initialize latex document with a template
    Init(InitArgs),

    /// Compile latex document
    Compile(CompileArgs),

    /// Clean temporary files
    Clean(CleanArgs),

    /// Show errors and warnings from the last compilation
    Log(LogArgs),

    /// Commands for managing templates
    Template(TemplateArgs),

    /// Alias for `template add`
    Add(TemplateAddArgs),

    /// Manage blatex configuration
    Config(ConfigArgs),
}

#[derive(Clone, clap::Args)]
pub struct InitArgs {
    /// Name of a template to use
    #[arg(short, long)]
    pub template: Option<String>,

    /// Main latex entrypoint
    #[arg(short, long)]
    pub main: Option<String>,
}

#[derive(Clone, clap::Args)]
pub struct CompileArgs {
    /// Entry point for the latex compiler
    #[clap(index = 1)]
    pub main_file: Option<String>,
}

#[derive(Clone, clap::Args)]
pub struct CleanArgs {
    /// Entry point for the latex compiler
    #[clap(index = 1)]
    pub main_file: Option<String>,
}

#[derive(Clone, clap::Args)]
pub struct LogArgs {
    /// Log file to show errors for
    #[clap(index = 1)]
    pub log_file: Option<String>,
}

#[derive(Clone, clap::Args)]
pub struct TemplateArgs {
    #[clap(subcommand)]
    pub template_command: TemplateCommand,
}

#[derive(Clone, clap::Args)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub config_command: ConfigCommand,

    /// Opterate on the global config instead of the local
    #[arg(short, long, default_value_t = false)]
    pub global: bool,
}

#[derive(Subcommand, Clone)]
pub enum TemplateCommand {
    // Add a local file or directory to templates
    Add(TemplateAddArgs),

    /// Add a git repository to templates
    AddRepo(TemplateAddRepoArgs),

    /// List templates
    List,
}

#[derive(Clone, clap::Args)]
pub struct TemplateAddArgs {
    /// The path to a zip-file or directory of zip-files
    #[arg(required = true)]
    pub paths: Vec<String>,

    /// Symlink instead of copying files to templates directory
    #[arg(long, default_value_t = false)]
    pub symlink: bool,

    /// Rename template or file or directory
    #[arg(short, long)]
    pub rename: Option<String>,

    /// Override existing templates
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

#[derive(Clone, clap::Args)]
pub struct TemplateAddRepoArgs {
    /// The URL to a repository
    #[clap(index = 1)]
    pub url: String,

    // Branch of the repository
    #[arg(short, long)]
    pub branch: Option<String>,

    /// The path to the template file or directory within the repository
    #[arg(short, long)]
    pub path: Option<String>,

    /// Rename template or file or directory
    #[arg(short, long)]
    pub rename: Option<String>,

    /// Override existing templates
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

#[derive(Subcommand, Clone)]
pub enum ConfigCommand {
    /// Create a local or global configuration file with the default options
    Create(ConfigCreateArgs),

    /// Dump the current configuration to stdout
    Show,
}

#[derive(Clone, clap::Args)]
pub struct ConfigCreateArgs {
    /// Override existing templates
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The root directory in the document folder structure; The directory containing the
    /// `.blatex.toml` file.
    pub root: PathBuf,

    /// The main entry point for the latex compiler
    pub main_file: PathBuf,

    /// Command for compiling document. \<main-file\> will be substituted with the `main_file`
    /// configuration field.
    pub compile_cmd: String,

    /// Command for cleaning temporary document files. \<main-file\> will be substituted with the `main_file`
    /// configuration field.
    pub clean_cmd: String,

    /// Directory for application data
    pub data_dir: PathBuf,

    /// Directory for storing templates
    pub templates_dir: PathBuf,

    /// Directory used for configuration of blatex
    pub config_file: PathBuf,

    /// Directory used for temporary files
    pub temp_dir: PathBuf,

    /// Remote templates and their options
    pub remote_templates: HashMap<String, RemoteTemplate>,
}

fn get_cwd() -> PathBuf {
    match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => exit_with_error!("Error getting current directory: {e}"),
    }
}

impl Default for Config {
    fn default() -> Self {
        let root = get_cwd();
        let proj_dirs = match ProjectDirs::from("com", "blatex", "blatex") {
            Some(dirs) => dirs,
            None => exit_with_error!("Could not determine application directories."),
        };
        let data_dir = proj_dirs.data_dir().to_path_buf();
        let templates_dir = data_dir.join("templates");
        let config_dir = proj_dirs.config_dir().join("blatex.toml");
        let temp_dir = proj_dirs.cache_dir().join("tmp");

        Config {
            root,
            data_dir,
            templates_dir,
            config_file: config_dir,
            temp_dir,
            main_file: PathBuf::from("main.tex"),
            compile_cmd: "pdflatex -shell-escape -interaction=nonstopmode <main-file>".to_string(),
            clean_cmd: "rm <main-stem>.aux <main-stem>.log".to_string(),
            remote_templates: HashMap::new(),
        }
    }
}

impl Config {
    pub fn override_some_fields(config: &mut Config, map: &Map<String, toml::Value>) {
        Self::override_pathbuf_if_some_string(&mut config.data_dir, map.get("data_dir"));
        Self::override_pathbuf_if_some_string(&mut config.templates_dir, map.get("templates_dir"));
        Self::override_pathbuf_if_some_string(&mut config.config_file, map.get("config_file"));
        Self::override_pathbuf_if_some_string(&mut config.temp_dir, map.get("temp_dir"));
        Self::override_pathbuf_if_some_string(&mut config.main_file, map.get("main_file"));
        Self::override_if_some_string(&mut config.compile_cmd, map.get("compile_cmd"));
        Self::override_if_some_string(&mut config.clean_cmd, map.get("clean_cmd"));
    }

    fn override_pathbuf_if_some_string(var: &mut PathBuf, value: Option<&toml::Value>) {
        if let Some(toml::Value::String(s)) = value {
            *var = PathBuf::from(s);
        }
    }

    fn override_if_some_string(var: &mut String, value: Option<&toml::Value>) {
        if let Some(toml::Value::String(s)) = value {
            *var = s.clone();
        }
    }

    fn parse_toml(s: &str) -> Map<String, toml::Value> {
        match toml::from_str(s) {
            Ok(c) => c,
            Err(e) => exit_with_error!("{e}"),
        }
    }

    pub fn new_global() -> Self {
        let mut config = Config::default();

        if config.config_file.is_file() {
            let global_toml = match fs::read_to_string(&config.config_file) {
                Ok(toml) => toml,
                Err(e) => {
                    eprintln!(
                        "Could not read global configuration file at '{}': {}",
                        config.config_file.display(),
                        e
                    );
                    return config;
                }
            };
            let global_config: Map<String, toml::Value> = Self::parse_toml(global_toml.as_str());

            if let Some(toml::Value::Table(table)) = global_config.get(REMOTE_TEMPLATES_OPTION) {
                for (name, value) in table.iter() {
                    let remote_template = match value {
                        toml::Value::String(url) => RemoteTemplate::from_url(url.clone()),
                        toml::Value::Table(fields) => {
                            let url = match fields.get("repo") {
                                Some(toml::Value::String(r)) => r.clone(),
                                Some(_) => {
                                    exit_with_error!("ERROR: Repository url must be string.")
                                }
                                None => exit_with_error!(
                                    "ERROR: Repository for '{}' is not defined.",
                                    name
                                ),
                            };
                            let path = match fields.get("path") {
                                Some(toml::Value::String(p)) => Some(PathBuf::from(p)),
                                _ => None,
                            };
                            let branch = match fields.get("branch") {
                                Some(toml::Value::String(b)) => Some(b.clone()),
                                _ => None,
                            };

                            let mut remote_config = Config::default();
                            Self::override_some_fields(&mut remote_config, fields);

                            RemoteTemplate::new(url, path, branch, remote_config)
                        }
                        _ => exit_with_error!(
                            "Error in remote template '{}'. Must be string or table of options.",
                            name
                        ),
                    };
                    config
                        .remote_templates
                        .insert(name.clone(), remote_template);
                }
            }

            Self::override_some_fields(&mut config, &global_config);
        }

        config
    }

    /// Returns (root, local_config_path)
    fn find_local_config(dir: &PathBuf) -> Option<(PathBuf, PathBuf)> {
        let config_path = dir.join(LOCAL_CONFIG_FILE);
        if config_path.exists() {
            Some((dir.to_owned(), config_path))
        } else {
            let parrent = dir.parent()?.to_path_buf();
            Self::find_local_config(&parrent)
        }
    }

    pub fn new_local(cwd: &PathBuf, provided_config_file: Option<PathBuf>) -> Self {
        let mut config = Config::new_global();

        let default_config = provided_config_file.is_none();

        let local_config_file = match provided_config_file {
            Some(p) => p,
            None => match Self::find_local_config(cwd) {
                Some((root, p)) => {
                    config.root = root;
                    p
                }
                None => {
                    // We cannot find a local config
                    return config;
                }
            },
        };

        if let Ok(toml) = fs::read_to_string(&local_config_file) {
            let local_config: Map<String, toml::Value> = Self::parse_toml(toml.as_str());
            Self::override_some_fields(&mut config, &local_config);
        } else if !default_config {
            eprintln!(
                "Could not read local config file `{}`. Skipping.",
                local_config_file.display()
            );
        }
        config
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteTemplate {
    pub url: String,
    pub path: Option<PathBuf>,
    pub branch: Option<String>,
    pub config: Config,
}

impl RemoteTemplate {
    pub fn new(url: String, path: Option<PathBuf>, branch: Option<String>, config: Config) -> Self {
        Self {
            url,
            path,
            branch,
            config,
        }
    }

    pub fn from_url(url: String) -> Self {
        Self::new(url, None, None, Config::default())
    }
}

#[derive(Clone)]
pub struct Opts {
    /// CLI arguments.
    pub args: Args,

    /// Configuration. Options that can be changed by configuration files.
    pub config: Config,

    /// Current Working directory
    pub cwd: PathBuf,
}

impl Opts {
    pub fn create() -> Self {
        let cwd = get_cwd();
        let args = Args::parse();
        let config = Config::new_local(&cwd, args.config_path.clone().map(PathBuf::from));
        Self { args, config, cwd }
    }
}

#[cfg(test)]
impl Opts {
    pub fn create_mock(args: Vec<&str>, config: Config, cwd: PathBuf) -> Self {
        let mut argv = vec!["blatex"];
        argv.extend(args);
        let args = Args::parse_from(argv);
        Self { args, config, cwd }
    }
}
