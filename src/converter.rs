use anyhow::{anyhow, Result};
use crate::ir::{ExactRecord, ExactRange, ExactDateTime, ExactDate, ExactTime, TimeZoneChoice, ExactEvent};
use icalendar as ical;
use icalendar::{Component, EventLike};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, prelude as cr, Timelike};
use chrono::{TimeZone,Local, Utc};
use chrono::LocalResult::Single;

impl ExactTime {
    pub fn to_chrono(self) -> Result<NaiveTime> {
        match NaiveTime::from_hms_opt(self.hour, self.minute, self.second) {
            None => {Err(anyhow!("Invalid time: {}:{}:{}", self.hour, self.minute, self.second))},
            Some(res) => {Ok(res)}
        }
    }
}

impl ExactDate {
    pub fn to_chrono(self) -> Result<NaiveDate> {
        match NaiveDate::from_ymd_opt(self.year, self.month, self.day) {
            None => {Err(anyhow!("Invalid date: {}-{}-{}", self.year, self.month, self.day))},
            Some(res) => {Ok(res)}
        }
    }
}

impl ExactDateTime{
    pub fn to_chrono(&self) -> Result<cr::DateTime<Utc>> {
        let baset = NaiveDateTime::new(self.date.to_chrono()?, self.time.to_chrono()?);
        match self.tz {
            TimeZoneChoice::Local => {
                let t = Local.from_local_datetime(&baset);
                match t {
                    Single(t) => Ok(t.with_timezone(&Utc)),
                    _ => Err(anyhow!("Error processing DateTime: {}", baset)),
                }
            },
            TimeZoneChoice::Utc => {
                Ok(Utc.from_utc_datetime(&baset))
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
}

impl ExactEvent {
    fn to_icalevent(&self) -> Result<ical::Event> {
        let mut calevent = ical::Event::new();
        calevent.summary(self.name.as_str());
        if let Some(notes) = self.notes.as_ref(){
            calevent.description(notes.as_str());
        }
        match &self.range{
            ExactRange::TimeRange(range) => {
                calevent = calevent
                    .starts(range.start.to_chrono()?)
                    .ends(range.end.to_chrono()?)
                    .done();
            },
            // ExactRange::Duration(duration) => {
            //     let shift = chrono::Duration::minutes(duration.duration as i64);
            //     calevent = calevent
            //         .starts(duration.start.to_chrono()?)
            //         .ends(duration.start.to_chrono()?+shift)
            //         .done();
            // },
            ExactRange::AllDay(date) => {
                calevent = calevent
                    .all_day(date.to_chrono()?)
                    .done();
            }
        }
        Ok(calevent)
    }
}

pub fn to_ical(records: Vec<ExactRecord>) -> String {
    let mut calendar = ical::Calendar::new();
    for record in records {
        if let ExactRecord::Event(event) = record {
            match event.to_icalevent() {
                Ok(calevent) => {
                    calendar.push(calevent);
                },
                Err(e) => {
                    eprintln!("Error processing event: {}", e);
                }
            }
        }
    }
    calendar = calendar.done();
    calendar.to_string()
}