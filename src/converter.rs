use crate::ir::{ExactRecord, ExactRange, ExactDateTime, ExactDate, ExactTime, TimeZoneChoice};
use icalendar as ical;
use icalendar::{Component, EventLike};
use chrono::{Datelike, prelude as cr, Timelike};
use chrono::{TimeZone,Local, Utc};

impl ExactTime {
    pub fn to_chrono(self) -> cr::NaiveTime {
        cr::NaiveTime::from_hms_opt(self.hour, self.minute, self.second).unwrap()
    }
}

impl ExactDate {
    pub fn to_chrono(self) -> cr::NaiveDate {
        // eprintln!("ExactDate::to_chrono: {}-{}-{}", self.year, self.month, self.day);
        cr::NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap()
    }
}

impl ExactDateTime{
    pub fn to_chrono(&self) -> cr::DateTime<Utc> {
        let date = self.date.to_chrono();
        let time = self.time.to_chrono();
        match self.tz {
            TimeZoneChoice::Local => {
                let t = Local.with_ymd_and_hms(date.year(), date.month(), date.day(), time.hour(), time.minute(), time.second()).unwrap();
            //     convert t to Utc
                t.with_timezone(&Utc)
            },
            TimeZoneChoice::Utc => Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), time.hour(), time.minute(), time.second()).unwrap(),
        }
    }
}

pub fn to_ical(records: Vec<ExactRecord>) -> String {
    let mut calendar = ical::Calendar::new();
    for record in records {
        if let ExactRecord::Event(event) = record {
            let mut calevent = ical::Event::new();
            calevent.summary(event.name.as_str());
            if let Some(notes) = event.notes.as_ref(){
                calevent.description(notes.as_str());
            }
            match &event.range{
                ExactRange::TimeRange(range) => {
                    calevent = calevent
                        .starts(range.start.to_chrono())
                        .ends(range.end.to_chrono())
                        .done();
                    // eprintln!("to_ical: {:?} {:?}", range.start, range.end);
                },
                ExactRange::Duration(duration) => {
                    let shift = chrono::Duration::minutes(duration.duration as i64);
                    calevent = calevent
                        .starts(duration.start.to_chrono())
                        .ends(duration.start.to_chrono()+shift)
                        .done();
                    // eprintln!("to_ical: {:?} {:?}", duration.start, duration.duration);
                },
                ExactRange::AllDay(date) => {
                    calevent = calevent
                        .all_day(date.to_chrono())
                        .done();
                    // eprintln!("to_ical: {:?}", date);
                }
            }
            // eprintln!("{:?}", calevent);
            calendar.push(calevent);
        }
    }
    calendar = calendar.done();
    calendar.to_string()
}