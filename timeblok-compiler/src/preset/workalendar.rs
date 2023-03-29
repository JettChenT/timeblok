use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use icalendar::Calendar;
use std::fs::create_dir_all;

use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, fs::File, io::Read};

use reqwest::Url;
use crate::utils::{download_file, get_dir};

fn download_workdays(country: &String) -> Result<()> {
    let url = format!(
            "https://raw.githubusercontent.com/JettChenT/workalendar-hub/main/workingdays/{}.txt",
            country
    );
    let dest = get_dir()?.join("workdays").join(format!("{}.txt", country));
    download_file(&url, dest, Some(format!("{} calendar", country).as_str()))
}

pub fn get_workdays(country: &String, new: bool) -> Result<Vec<NaiveDate>> {
    let fpath = get_dir()?.join("workdays").join(format!("{}.txt", country));
    if new || !fpath.exists() {
        download_workdays(country)?;
    }
    let mut file = File::open(fpath)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    // split by new line and read by YYYY-MM-DD format
    let mut res = Vec::new();
    for l in contents.lines() {
        if let Ok(date) = NaiveDate::parse_from_str(l, "%Y-%m-%d") {
            res.push(date);
        }
    }
    Ok(res)
}

fn download_holiday(country: &String) -> Result<()> {
    let url = Url::parse(
        format!(
            "https://raw.githubusercontent.com/JettChenT/workalendar-hub/main/holidays-ics/{}.ics",
            country
        )
        .as_str(),
    )?;
    eprintln!("downloading {} holidays calendar...", country);
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(anyhow!("Cannot download calendar: {}", response.text()?));
    }
    let dest = {
        let fpath = get_dir()?.join(format!("{}.ics", country));
        File::create(&fpath)?;
        fpath
    };
    // write contents of response to dest
    fs::write(dest, response.bytes()?)?;
    Ok(())
}

pub fn get_holiday(country: &String, new: bool) -> Result<Calendar> {
    let fpath = get_dir()?.join(format!("{}.ics", country));
    if new || !fpath.exists() {
        download_holiday(country)?;
    }
    let mut file = File::open(fpath)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    match Calendar::from_str(&contents) {
        Ok(cal) => Ok(cal),
        Err(e) => Err(anyhow!("Cannot parse calendar: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_holiday() -> Result<()> {
        let cal = get_holiday(&"US".to_string(), true)?;
        Ok(())
    }

    #[test]
    fn test_empty_holiday() -> Result<()> {
        assert!(get_holiday(&"XX".to_string(), true).is_err());
        Ok(())
    }
}
