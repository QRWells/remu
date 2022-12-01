use clap::{arg, command, Parser};
use log::*;
use object::{Object, ObjectSection};
use std::fs;

/// Simulator for pipeline processors
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File name to execute
    #[arg()]
    file: String,
}

fn main() {
    let args = Args::parse();

    println!("Hello, {}!", args.file);
    // open the file as binary
    if let Ok(file) = fs::read(args.file) {
        // parse the file as an ELF object
        if let Ok(file) = object::File::parse(&*file) {
            // iterate over the sections
            for section in file.sections() {
                // print the section name and size
                info!(
                    "{}: {}",
                    section.name().unwrap_or("<unnamed>"),
                    section.size()
                );
            }
        } else {
            error!("Error parsing file");
        }
    } else {
        error!("Error opening file");
    }
}
