use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Opts {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize latex document with a template
    Init,

    /// Compile latex document
    Compile,

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
