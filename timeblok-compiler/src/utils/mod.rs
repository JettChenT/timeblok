use std::fs;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
#[cfg(not(target_family = "wasm"))]
use reqwest::Url;
use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use crate::ir::{ExactDateTime, ExactDate, ExactTime, TimeZoneChoice};
use std::time::SystemTime;
use chrono::Local;
use chrono::{prelude as cr, Datelike, Timelike};

#[cfg(target_family = "wasm")]
use web_sys::{Request, RequestInit, RequestMode, Response};
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_family = "wasm"))]
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

#[cfg(target_family = "wasm")]
pub async fn download_file_wasm(filename: &str, dest: PathBuf, display_name: Option<&str>) -> Result<()> {
    // Create a Request object with the URL of the file to download.

    use wasm_bindgen_futures::JsFuture;
    let request = Request::new_with_str_and_init(
        filename,
        &RequestInit::new()
            .method("GET")
            .mode(RequestMode::Cors)
    ).unwrap();

   let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();
    let body = resp.body().unwrap().as_string().unwrap();
    fs::write(dest, body)?;
    Ok(())
}

pub enum Dirs{
    Project,
    Data,
    Cache,
    Base,
}

// TODO: update this to match wasm implementation
#[cfg(not(target_family = "wasm"))]
pub fn get_dir() -> Result<PathBuf> {
    if let Some(dir) = ProjectDirs::from("", "", "timeblok") {
        let data_dir = dir.data_dir().join("workalendar");
        create_dir_all(&data_dir)?;
        Ok(data_dir)
    } else {
        Err(anyhow!("Cannot find project directory"))
    }
}

#[cfg(target_family = "wasm")]
pub fn get_dir(dir: Dirs, subdir: Option<&String>) -> Result<PathBuf> {
    use Dirs::*;
    let mut path = match dir {
        Project => PathBuf::from("project"),
        Data => PathBuf::from("data"),
        Cache => PathBuf::from("cache"),
        Base => PathBuf::from("base"),
    };
    if let Some(subdir) = subdir {
        path.push(subdir);
    }
    Ok(path)
}

impl ExactDateTime{
    pub fn from_system_time(base_t: SystemTime) -> Self{
        let base_time: cr::DateTime<Local> = base_t.into();
        Self{
            date: {
                let date = base_time.date_naive();
                ExactDate {
                    year: date.year(),
                    month: date.month(),
                    day: date.day(),
                }
            },
            time: {
                ExactTime {
                    hour: 0,
                    minute: 0,
                    second: 0,
                }
            },
            tz: TimeZoneChoice::Local,
        }
    }
}
