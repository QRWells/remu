#[allow(dead_code)]
pub mod arch;
pub mod board;
pub mod bus;
pub mod cpu;
pub mod mem;

use clap::{arg, command, Parser};
use env_logger::Env;
use goblin::Object;
use std::{fs, path::Path};

#[macro_use]
extern crate log;

/// Simulator for pipeline processors
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File name to execute
    #[arg()]
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup logging to output all logs
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // open the file as binary
    let path = Path::new(&args.file);
    let buffer = fs::read(path)?;
    match Object::parse(&buffer)? {
        Object::Elf(elf) => {
            info!("elf: {:#?}", &elf.header);
        }
        _ => {
            error!("Unsupported file format");
        }
    }
    Ok(())
}
