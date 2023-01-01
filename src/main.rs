extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs;
use std::io::Write;
use pest::Parser;
use crate::parser::{parse_file, Rule};
use anyhow::Result;

mod parser;
mod args;
mod ir;
mod resolver;
mod converter;

use parser::BlokParser;
use resolver::resolve;
use converter::to_ical;

fn main() {
    let args = args::parse();
    match try_main(args) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn try_main(args: args::Args) -> Result<()> {
    let metadata = fs::metadata(&args.infile)?;
    let created = metadata.created()?;
    let file = fs::read_to_string(&args.infile).unwrap();
    let file = BlokParser::parse(Rule::FILE, &file)
        .expect("Unsuccessful parse")
        .next()
        .unwrap();

    let records= parse_file(file)?;
    let resolved = resolve(records, created);
    let converted = to_ical(resolved);
    match &args.outfile {
        Some(path) => {
            let mut file = fs::File::create(path)?;
            file.write_all(converted.as_bytes())?;
        }
        _ => {
            println!("{}", converted);
        }
    }
    Ok(())
}