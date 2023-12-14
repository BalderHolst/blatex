#![allow(dead_code)]

use clap::CommandFactory;

#[path = "src/opts.rs"]
mod opts;

#[path = "src/utils.rs"]
mod utils;

#[path = "src/config.rs"]
mod config;

fn main() -> std::io::Result<()> {
    let out_dir = std::path::PathBuf::from("man");

    let cmd = opts::Args::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    std::fs::write(out_dir.join("blatex.1"), buffer)?;

    Ok(())
}
