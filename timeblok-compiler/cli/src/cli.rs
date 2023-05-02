use anyhow::{anyhow, Result};
use clap::ValueEnum;
use directories::{BaseDirs, ProjectDirs};

use std::fs;
use std::io::Write;

use timeblok::{tb_to_records, records_to_resolved, resolved_to_ical, ir::ExactDateTime, resolved_to_csv};

use crate::args::{parse, Args, OutputTypes};

pub fn main() {
    let args = parse();
    match try_main(args) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn handle_infile(infile: Option<String>, new: bool) -> Result<String> {
    match infile {
        Some(s) => Ok(s),
        None => match new {
            true => {
                let cur_date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let template = format!("{}\n", cur_date);
                let edited = edit::edit(template)?;
                if let Some(dir) = ProjectDirs::from("", "", "timeblok") {
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
            }
            false => Err(anyhow!("No input file specified")),
        },
    }
}

fn try_main(args: Args) -> Result<()> {
    let infile = handle_infile(args.infile, args.new)?;
    let metadata = fs::metadata(&infile)?;
    let created = metadata.created()?;
    let file = fs::read_to_string(&infile)?;
    let records = tb_to_records(&file)?;
    if args.print {
        println!("{:#?}", records);
    }
    if args.parse_only {
        return Ok(());
    }
    let resolved = records_to_resolved(records, ExactDateTime::from_system_time(created))?;
    let ext = match &args.format{
        Some(s) => s.to_owned(),
        None => {
            if let Some(path) = &args.outfile{
                let split: Vec<&str> = path.split(".").collect();
                let tmp = split.last().unwrap();
                OutputTypes::from_str(tmp, true)
                    .unwrap_or(OutputTypes::Ics)
            }else{OutputTypes::Ics}
        }
    };
    let converted = match ext {
        OutputTypes::Csv => resolved_to_csv(resolved)?,
        OutputTypes::Ics => resolved_to_ical(resolved)?,
    };
    match &args.outfile {
        Some(path) => {
            let mut file = fs::File::create(path)?;
            file.write_all(converted.as_bytes())?;
            if args.open {
                open::that(path)?;
            }
        }
        _ => {
            if args.print {
                println!("{}", converted);
            }
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
