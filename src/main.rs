extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate core;

use crate::parser::{parse_file, Rule};
use anyhow::{Result, anyhow};
use directories::{BaseDirs, ProjectDirs};
use edit::edit_file;
use pest::Parser;
use std::fs;
use std::io::Write;

mod args;
mod converter;
mod environment;
mod ir;
mod output;
mod parser;
mod resolver;
mod preset;

use converter::to_ical;
use parser::BlokParser;
use resolver::resolve;

fn main() {
    let args = args::parse();
    match try_main(args) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn handle_infile(infile: Option<String>, new:bool) -> Result<String>{
    match infile {
        Some(s) => Ok(s),
        None => match new {
            true => {
                let cur_date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let template = format!("{}\n", cur_date);
                let edited = edit::edit(template)?;
                if let Some(dir) = ProjectDirs::from("", "", "timeblok"){
                    let data_dir = dir.data_dir().join("bloks");
                    fs::create_dir_all(&data_dir)?;
                    let filename = format!("{}.blok", cur_date);
                    let path = data_dir.join(filename);
                    fs::write(&path, edited)?;
                    let pathstr = path.to_string_lossy().to_string();
                    eprintln!("File created at {}", &pathstr);
                    Ok(pathstr)
                } else {
                    Err(anyhow!("Could not get data directory"))
                }
            },
            false => Err(anyhow!("No input file specified")),
        },
    }
}

fn try_main(args: args::Args) -> Result<()> {
    let infile = handle_infile(args.infile, args.new)?;
    let metadata = fs::metadata(&infile)?;
    let created = metadata.created()?;
    let file = fs::read_to_string(&infile).unwrap();
    let file = BlokParser::parse(Rule::FILE, &file)
        .expect("Unsuccessful parse")
        .next()
        .unwrap();

    let records = parse_file(file)?;
    if args.print {
        println!("{:#?}", records);
    }
    if args.parse_only {
        return Ok(());
    }
    let resolved = resolve(records, created);
    let converted = to_ical(resolved);
    match &args.outfile {
        Some(path) => {
            let mut file = fs::File::create(path)?;
            file.write_all(converted.as_bytes())?;
            if args.open {
                open::that(path)?;
            }
        }
        _ => {
            if args.print{println!("{}", converted);}
            if args.open {
                // Save file in temporary directory and open it
                let base_dirs = BaseDirs::new().expect("Could not get base directories");
                let temp_dir = base_dirs.cache_dir();
                let path = temp_dir.join(".blok.ics");
                let mut file = fs::File::create(&path)?;
                file.write_all(converted.as_bytes())?;
                open::that(&path)?;
            }
        }
    }
    Ok(())
}
