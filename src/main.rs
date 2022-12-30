extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs;
use std::io::Write;
use pest::Parser;
use crate::parser::{parse_file, parse_record, Rule};
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
    let file = std::fs::read_to_string(&args.infile).unwrap();
    let file = BlokParser::parse(Rule::FILE, &file)
        .expect("Unsuccessful parse")
        .next()
        .unwrap();

    let records= parse_file(file)?;
    // for record in records {
    //     println!("{:?}", record);
    // }
    // println!("------------");
    let resolved = resolve(records, created);
    // for record in resolved {
    //     println!("{:?}", record);
    // }
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