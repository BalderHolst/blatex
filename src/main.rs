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

#[cfg(test)]
mod tests;

fn main() {
    let opts = Opts::create();
    run(opts);
}

fn run(opts: Opts) {
    match opts.args.command {
        Command::Init(args) => init::init(opts.cwd, opts.config, args),
        Command::Compile(args) => compile::compile(opts.config, args),
        Command::Clean(args) => clean::clean(opts.config, args),
        Command::Log(args) => log::print_log(
            opts.config.root,
            &match &args.log_file {
                Some(s) => PathBuf::from(s),
                None => opts.config.main_file,
            },
        ),
        Command::Add(args) => templates::add_paths(opts.cwd, opts.config, args),
        Command::Template(args) => match args.template_command {
            opts::TemplateCommand::Add(args) => templates::add_paths(opts.cwd, opts.config, args),
            opts::TemplateCommand::AddRepo(args) => {
                templates::add_repo(opts.cwd, opts.config, args)
            }
            opts::TemplateCommand::List => templates::list_templates(opts.config),
        },
        Command::Config(args) => match &args.config_command {
            opts::ConfigCommand::Create(create_args) => {
                config::create(&opts.cwd, args.global, create_args, &opts.config)
            }
            opts::ConfigCommand::Show => config::show(opts.config, args.global),
        },
    }
}
