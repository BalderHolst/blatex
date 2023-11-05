use std::{collections::HashMap, fs, path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use toml::map::Map;

use crate::config::LOCAL_CONFIG_FILE;

const REMOTE_TEMPLATES_OPTION: &str = "templates";

#[derive(Parser, Clone)]
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
    Templates(TemplateArgs),

    /// Manage blatex configuration
    Config(ConfigArgs),
}

#[derive(Clone, clap::Args)]
pub struct InitArgs {
    /// Name of a template to use
    #[arg(short, long)]
    pub template: Option<String>,
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

    /// The path to the template file or directory within the repository
    #[arg(short, long)]
    pub path: Option<String>,

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
    /// The main entry point for the latex compiler
    pub main_file: String,

    /// Command for compiling document. <main-file> will be substituted with the `main_file`
    /// configuration field.
    pub compile_cmd: String,

    /// Command for cleaning temporary document files. <main-file> will be substituted with the `main_file`
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
            main_file: "main.tex".to_string(),
            compile_cmd:
                "latexmk -pdf -bibtex-cond -shell-escape -interaction=nonstopmode <main-file>"
                    .to_string(),
            clean_cmd: "latexmk -c <main-file>".to_string(),
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
        Self::override_if_some_string(&mut config.main_file, map.get("main_file"));
        Self::override_if_some_string(&mut config.compile_cmd, map.get("compile_cmd"));
        Self::override_if_some_string(&mut config.clean_cmd, map.get("clean_cmd"));
    }

    fn override_pathbuf_if_some_string(var: &mut PathBuf, value: Option<&toml::Value>) {
        if let Some(toml_value) = value {
            if let toml::Value::String(s) = toml_value {
                *var = PathBuf::from(s);
            }
        }
    }

    fn override_if_some_string(var: &mut String, value: Option<&toml::Value>) {
        if let Some(toml_value) = value {
            if let toml::Value::String(s) = toml_value {
                *var = s.clone();
            }
        }
    }

    pub fn new_global() -> Self {
        let mut config = Config::default();

        if config.config_file.is_file() {
            let global_toml = fs::read_to_string(&config.config_file).unwrap();
            let global_config: Map<String, toml::Value> =
                toml::from_str(global_toml.as_str()).unwrap();

            if let Some(toml::Value::Table(table)) = global_config.get(REMOTE_TEMPLATES_OPTION) {
                for (name, value) in table.iter() {
                    let remote_template = match value {
                        toml::Value::String(url) => RemoteTemplate::from_url(url.clone()),
                        toml::Value::Table(fields) => {
                            let url = match fields.get("repo") {
                                Some(toml::Value::String(r)) => r.clone(),
                                Some(_) => {
                                    eprintln!("ERROR: Repository url must be string.");
                                    exit(1)
                                }
                                None => {
                                    eprintln!("ERROR: Repository for '{}' is not defined.", name);
                                    exit(1)
                                }
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
                        _ => {
                            eprintln!("Error in remote template '{}'. Must be string or table of options.", name);
                            exit(1)
                        }
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

    pub fn new_local(cwd: &PathBuf, local_config_file: Option<PathBuf>) -> Self {
        let mut config = Config::new_global();

        let default_config = local_config_file.is_none();

        let local_config_file = local_config_file.unwrap_or(cwd.join(LOCAL_CONFIG_FILE));

        if let Ok(toml) = fs::read_to_string(&local_config_file) {
            let local_config: Map<String, toml::Value> = toml::from_str(toml.as_str()).unwrap();
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
        Self { url, path, branch, config }
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
        let cwd = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error getting current directory: {e}");
                exit(1);
            }
        };
        let args = Args::parse();
        let config = Config::new_local(&cwd, args.config_path.clone().map(|s| PathBuf::from(s)));
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
