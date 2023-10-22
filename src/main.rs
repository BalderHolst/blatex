mod clean;
mod compile;
mod config;
mod init;
mod log;
mod opts;
mod templates;
mod utils;

use std::path::PathBuf;

use opts::{Command, Opts};

fn main() {
    let opts = Opts::create();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match opts.args.command {
        Command::Init { template } => init::init(opts.config.templates_dir, template),
        Command::Compile {
            main_file: cli_main_file,
        } => compile::compile(
            opts.config.compile_cmd,
            cli_main_file.unwrap_or(opts.config.main_file),
        ),
        Command::Clean {
            main_file: cli_main_file,
        } => clean::clean(
            opts.config.clean_cmd,
            cli_main_file.unwrap_or(opts.config.main_file),
        ),
        Command::Log => log::print_log(opts.config.main_file),
        Command::Templates { template_command } => match template_command {
            opts::TemplateCommand::Add {
                path,
                symlink,
                force,
            } => templates::add_path(
                PathBuf::from(path),
                symlink,
                opts.config.templates_dir,
                force,
            ),
            opts::TemplateCommand::AddRepo { url, path, force } => templates::add_repo(
                url,
                path,
                opts.config.templates_dir,
                force,
                opts.config.temp_dir,
            ),
            opts::TemplateCommand::List => templates::list_templates(opts.config.templates_dir),
        },
        Command::Config {
            config_command,
            global,
        } => match config_command {
            opts::ConfigCommand::Create { force } => config::create(global, force),
            opts::ConfigCommand::Show => config::show(opts.config, global),
        },
    }
}
