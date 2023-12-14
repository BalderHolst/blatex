#![allow(dead_code)]

use clap::{Command, CommandFactory};

#[path = "src/opts.rs"]
mod opts;

#[path = "src/utils.rs"]
mod utils;

#[path = "src/config.rs"]
mod config;

fn main() -> std::io::Result<()> {
    let out_man_dir = env!("CARGO_MANIFEST_DIR").to_string() + "/man";

    create_man_page("blatex.1", opts::Args::command(), out_man_dir.as_str())?;

    Ok(())
}

fn create_man_page(filename: &str, cmd: Command, out_dir: &str) -> std::io::Result<()> {
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    std::fs::write(out_dir.to_string() + "/" + filename, buffer)?;

    Ok(())
}
