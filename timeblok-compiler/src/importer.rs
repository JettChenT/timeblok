use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::{
    environment::Environment,
    ir::{filter::Filter, Date, ExactDate},
    resolver::resolve_date,
};
use chrono::NaiveDate;
use icalendar::{Calendar, Component};
use crate::ir::{ExactDateTime, ExactEvent, ExactRange, ExactRecord, ExactTime, ExactTimeRange, Todo};
use anyhow::{Result, anyhow};
use crate::utils::get_dir;
#[cfg(not(target_family = "wasm"))]
use crate::utils::download_file;

#[derive(Debug, Clone)]
pub struct SetFilter {
    dates: HashSet<ExactDate>,
}

impl Filter<Date> for SetFilter {
    fn check(&self, value: &Date, env: Option<&Environment>) -> bool {
        self.dates
            .contains(&resolve_date(value, env.unwrap()).unwrap())
    }
}

impl SetFilter {
    pub fn from_naive_dates(dates: impl IntoIterator<Item = NaiveDate>) -> Self {
        let mut set = HashSet::new();
        for date in dates {
            set.insert(ExactDate::from_naive(date));
        }
        SetFilter { dates: set }
    }

    pub fn from_ics(cal: &Calendar) -> Self {
        let mut dates = HashSet::new();
        for c in cal.iter() {
            if let Some(event) = c.as_event() {
                match (event.get_start(), event.get_end()) {
                    (Some(st), Some(nd)) => {
                        let est = ExactDate::from_date_perhaps_time(st).to_naive();
                        let end = ExactDate::from_date_perhaps_time(nd).to_naive();
                        for i in 0..=(end - est).num_days() {
                            dates.insert(ExactDate::from_naive(est + chrono::Duration::days(i)));
                        }
                    }
                    (Some(st), None) => {
                        let est = ExactDate::from_date_perhaps_time(st).to_naive();
                        dates.insert(ExactDate::from_naive(est));
                    }
                    (None, Some(nd)) => {
                        let end = ExactDate::from_date_perhaps_time(nd).to_naive();
                        dates.insert(ExactDate::from_naive(end));
                    }
                    (_, _) => {}
                }
            }
        }
        Self { dates }
    }
}


#[cfg(not(target_family = "wasm"))]
pub fn import_ics(url: &String) -> Result<Calendar>{
    let contents = if url.starts_with("http"){
        let loc = get_dir()?.join("ics").join(&url);
        download_file(url, loc.clone(), None)?;
        let mut contents = String::new();
        File::open(loc)?.read_to_string(&mut contents)?;
        contents
    } else {
        let mut contents = String::new();
        File::open(PathBuf::from_str(url.as_str())?)?.read_to_string(&mut contents)?;
        contents
    };
    match Calendar::from_str(&contents) {
        Ok(cal) => Ok(cal),
        Err(e) => Err(anyhow!(e))
    }
}

#[cfg(target_family = "wasm")]
pub fn import_ics(source: &String) -> Result<Calendar>{
    match Calendar::from_str(source) {
        Ok(cal) => Ok(cal),
        Err(e) => Err(anyhow!(e))
    }
}

pub fn ics_to_records(cal: &Calendar) -> Vec<ExactRecord>{
    let mut records = vec![];
    for c in cal.iter(){
        if let Some(event) = c.as_event() {
            let range  = match (event.get_start(), event.get_end()) {
                (Some(st), Some(nd)) => {
                    let est = ExactDateTime::from_date_perhaps_time(st);
                    let end  = ExactDateTime::from_date_perhaps_time(nd);
                    ExactRange::TimeRange(ExactTimeRange{
                        start: est,
                        end
                    })
                }
                (Some(st), None) => {
                    let est = ExactDate::from_date_perhaps_time(st);
                    ExactRange::AllDay(est)
                }
                (None, Some(nd)) => {
                    let end = ExactDate::from_date_perhaps_time(nd);
                    ExactRange::AllDay(end)
                }
                (_, _) => {continue;}
            };
            records.push(ExactRecord::Event(ExactEvent{
                range,
                name: event.get_summary().unwrap_or("").to_string(),
                notes: event.get_description().map(|s| s.to_string())
            }))
        }
        if let Some(td) = c.as_todo(){
            records.push(ExactRecord::Todo(Todo{
                name: td.get_summary().unwrap_or("").to_string(),
                due: None,
                status: td.get_status().unwrap_or(icalendar::TodoStatus::NeedsAction),
            }))
        }
    }
    records
}
