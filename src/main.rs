mod clean;
mod cli;
mod compile;
mod init;
mod templates;
mod config;

use std::path::PathBuf;

use cli::{Command, Opts};

fn main() {
    let opts = Opts::create();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match opts.args.command {
        Command::Init { template } => init::init(opts.config.templates_dir, template),
        Command::Compile { main_file } => compile::compile(main_file),
        Command::Clean { main_file } => clean::clean(main_file),
        Command::Log => todo!(),
        Command::Templates { template_command } => match template_command {
            cli::TemplateCommand::Add {
                path,
                symlink,
                force,
            } => templates::add_path(PathBuf::from(path), symlink, opts.config.templates_dir, force),
            cli::TemplateCommand::AddRepo { url, path, force } => {
                templates::add_repo(url, path, opts.config.templates_dir, force, opts.config.temp_dir)
            }
            cli::TemplateCommand::List => templates::list_templates(opts.config.templates_dir),
        },
        Command::Config { config_command, global } => match config_command {
            cli::ConfigCommand::Create { force } => config::create(global, force),
            cli::ConfigCommand::Show => config::show(opts.config, global),
        }
    }
}
