use std::path::PathBuf;

use clap::{Parser, Subcommand};
use directories::ProjectDirs;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize latex document with a template
    Init,

    /// Compile latex document
    Compile {

        #[clap(index = 1)]
        file: Option<String>,
    },

    /// Clean temporary files
    Clean,

    /// Show errors and warnings from the last compilation
    Errors,

    /// Commands for managing templates
    Templates {
        #[clap(subcommand)]
        template_command: TemplateCommand,
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
    },

    /// List templates
    List,
}

pub struct Opts {
    /// CLI arguments
    pub args: Args,

    pub data_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Opts {
    pub fn create() -> Self {
        let args = Args::parse();
        let proj_dirs = ProjectDirs::from("com", "blatex", "blatex").unwrap();
        let data_dir = proj_dirs.data_dir().to_path_buf();
        let templates_dir = data_dir.join("templates");
        let config_dir = proj_dirs.config_dir().to_path_buf();
        Self {
            args,
            data_dir,
            templates_dir,
            config_dir,
        }
    }
}
