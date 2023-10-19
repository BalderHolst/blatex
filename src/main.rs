mod cli;
mod templates;

use cli::{Command, Opts};
use templates::add_path;

fn main() {
    let opts = Opts::create();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match opts.args.command {
        Command::Init => todo!(),
        Command::Compile => todo!(),
        Command::Clean => todo!(),
        Command::Errors => todo!(),
        Command::Templates { template_command } => match template_command {
            cli::TemplateCommand::Add {
                path,
                symlink,
                force,
            } => add_path(path, symlink, opts.templates_dir, force),
            cli::TemplateCommand::AddRepo { url, path } => todo!(),
            cli::TemplateCommand::List => todo!(),
        },
    }
}
