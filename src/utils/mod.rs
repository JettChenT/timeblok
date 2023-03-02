use std::fs;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use reqwest::Url;
use anyhow::{anyhow, Result};
use directories::ProjectDirs;

pub fn download_file(filename:&str, dest:PathBuf, display_name: Option<&str>) -> Result<()>{
    let url = Url::parse(filename)?;
    let name = match display_name {
        Some(name) => name,
        None => filename,
    };
    eprintln!("downloading {}...", name);
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(anyhow!("Cannot download {}: {}", name, response.text()?));
    }
    // write contents of response to dest
    fs::create_dir_all(&dest.parent().unwrap())?;
    File::create(&dest)?;
    fs::write(&dest, response.bytes()?)?;
    println!("downloaded {} to {}", name, &dest.display());
    Ok(())
}

pub fn get_dir() -> Result<PathBuf> {
    if let Some(dir) = ProjectDirs::from("", "", "timeblok") {
        let data_dir = dir.data_dir().join("workalendar");
        create_dir_all(&data_dir)?;
        Ok(data_dir)
    } else {
        Err(anyhow!("Cannot find project directory"))
    }
}