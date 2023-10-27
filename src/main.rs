mod clean;
mod compile;
mod config;
mod init;
mod log;
mod opts;
mod templates;
mod utils;

use opts::{Command, Opts};

fn main() {
    let opts = Opts::create();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match opts.args.command {
        Command::Init { template } => init::init(opts.config, template),
        Command::Compile {
            main_file: cli_main_file,
        } => {
            let main_file = cli_main_file.unwrap_or(opts.config.main_file.clone());
            compile::compile(
                    opts.config,
                    &main_file,
                )
        },
        Command::Clean {
            main_file: cli_main_file,
        } => {
            let main_file = cli_main_file.unwrap_or(opts.config.main_file.clone());
            clean::clean(
                    opts.config,
                    &main_file,
                )
        },
        Command::Log { log_file } => log::print_log(&log_file.unwrap_or(opts.config.main_file)),
        Command::Templates { template_command } => match template_command {
            opts::TemplateCommand::Add {
                paths,
                symlink,
                force,
            } => templates::add_paths(opts.config, paths, symlink, force),
            opts::TemplateCommand::AddRepo { url, path, force } => templates::add_repo(
                opts.config,
                url,
                path,
                force,
            ),
            opts::TemplateCommand::List => templates::list_templates(opts.config),
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
