mod cli;

use clap::Parser;
use cli::{Opts, Commands};

fn main() {
    let cli = Opts::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Init => todo!(),
        Commands::Compile => todo!(),
        Commands::Clean => todo!(),
        Commands::Errors => todo!(),
        Commands::Templates { template_command } => todo!(),

    }
}
