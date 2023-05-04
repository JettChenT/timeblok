use std::char::CharTryFromError;

use crate::ir::{
    ExactDate, ExactDateTime, ExactEvent, ExactRange, ExactRecord, ExactTime, TimeZoneChoice,
};
use anyhow::{anyhow, Result};
use chrono::LocalResult::{Single, self};
use chrono::{prelude as cr, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use chrono::{Local, TimeZone, Utc};
use icalendar as ical;
use icalendar::{Component, EventLike};
use uuid::Uuid;

impl ExactTime {
    pub fn to_chrono(self) -> Result<NaiveTime> {
        match NaiveTime::from_hms_opt(self.hour, self.minute, self.second) {
            None => Err(anyhow!(
                "Invalid time: {}:{}:{}",
                self.hour,
                self.minute,
                self.second
            )),
            Some(res) => Ok(res),
        }
    }
}

impl ExactDate {
    pub fn to_chrono(self) -> Result<NaiveDate> {
        match NaiveDate::from_ymd_opt(self.year, self.month, self.day) {
            None => Err(anyhow!(
                "Invalid date: {}-{}-{}",
                self.year,
                self.month,
                self.day
            )),
            Some(res) => Ok(res),
        }
    }
}

impl ExactDateTime {
    pub fn to_chrono(&self) -> Result<cr::DateTime<Utc>> {
        let baset = NaiveDateTime::new(self.date.to_chrono()?, self.time.to_chrono()?);
        match self.tz {
            TimeZoneChoice::Local => {
                let t = Local.from_local_datetime(&baset);
                match t {
                    Single(t) => Ok(t.with_timezone(&Utc)),
                    _ => Err(anyhow!("Error processing DateTime: {}", baset)),
                }
            }
            TimeZoneChoice::Utc => Ok(Utc.from_utc_datetime(&baset)),
            TimeZoneChoice::Offset(o) => {
                let t = o.from_local_datetime(&baset);
                match t {
                    Single(t) => Ok(t.with_timezone(&Utc)),
                    _ => Err(anyhow!("Error processing DateTime: {}", baset)),
                }
            }
        }
    }

    pub fn from_chrono(t: cr::DateTime<Utc>) -> Self {
        ExactDateTime {
            date: ExactDate {
                year: t.year(),
                month: t.month(),
                day: t.day(),
            },
            time: ExactTime {
                hour: t.hour(),
                minute: t.minute(),
                second: t.second(),
            },
            tz: TimeZoneChoice::Utc,
        }
    }

    pub fn from_timestamp(timestamp: i64) -> Option<Self> {
        match Utc.timestamp_millis_opt(timestamp) {
            LocalResult::None => None,
            LocalResult::Ambiguous(_, _) => None,
            LocalResult::Single(t) => Some(
                Self { time: ExactTime::from_hms(0, 0, 0), ..Self::from_chrono(t)}
            ),
        }
    }
}

impl ExactEvent {
    fn to_icalevent(&self, key: Option<String>, tsmp: Option<chrono::DateTime<Utc>>) -> Result<ical::Event> {
        let mut calevent = ical::Event::new();
        calevent.summary(self.name.as_str());
        if let Some(notes) = self.notes.as_ref() {
            calevent.description(notes.as_str());
        }
        if let Some(s) = key {
            calevent.uid(Uuid::new_v3(&Uuid::NAMESPACE_URL, s.as_bytes()).to_string().as_str());
        }
        if let Some(tsmp)=tsmp{
            calevent.timestamp(tsmp);
        }
        match &self.range {
            ExactRange::TimeRange(range) => {
                calevent = calevent
                    .starts(range.start.to_chrono()?)
                    .ends(range.end.to_chrono()?)
                    .done();
            }
            ExactRange::AllDay(date) => {
                calevent = calevent.all_day(date.to_chrono()?).done();
            }
        }
        Ok(calevent)
    }
}

pub fn to_ical(records: Vec<ExactRecord>, deterministic_tsmp: Option<chrono::DateTime<Utc>>) -> String {
    let mut calendar = ical::Calendar::new();
    for (i,record) in records.iter().enumerate() {
        let key = if deterministic_tsmp.is_some() {
            Some(i.to_string())
        } else {
            None
        };
        
        match record {
            ExactRecord::Event(event) => {
                match event.to_icalevent(key, deterministic_tsmp) {
                    Ok(calevent) => {
                        calendar.push(calevent);
                    }
                    Err(e) => {
                        eprintln!("Error processing event: {}", e);
                    }
                }
            }
            ExactRecord::Note(_) => {}
            ExactRecord::Todo(t) => {
                match t.to_ical(key, deterministic_tsmp) {
                    Ok(caltodo) => {
                        calendar.push(caltodo);
                    }
                    Err(e) => {
                        eprintln!("Error processing todo: {}", e);
                    }
                }
            }
        }
    }
    calendar = calendar.done();
    calendar.to_string()
}

pub fn to_csv(records: Vec<ExactRecord>) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["timerange", "event"])?;
    for (i,record) in records.iter().enumerate() {
        if let ExactRecord::Event(event) = record {
            wtr.write_record([event.range.to_string().as_str(), event.name.as_str()])?;
        }
    }
    Ok(String::from_utf8(wtr.into_inner()?)?)
}

// test
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use anyhow::Result;

    #[test]
    fn test_timestamp() -> Result<()>{
        let timestamp = 1680533997811;
        if let Some(res) = ExactDateTime::from_timestamp(timestamp){
            assert_eq!(res.date.year, 2023);
            assert_eq!(res.date.month, 4);
            assert_eq!(res.date.day, 3);
            Ok(())
        } else {
            Err(anyhow::anyhow!("failed to parse timestamp"))
        }
    }
}