use crate::filter::Filter;
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum NumVal {
    Number(i64),
    Unsure,
}

#[derive(Debug)]
pub struct NumRange {
    pub start: NumVal,
    pub end: NumVal,
}

#[derive(Debug)]
pub enum Range {
    TimeRange(TimeRange),
    Duration(Duration),
    AllDay(Date),
}

#[derive(Debug)]
pub enum ExactRange {
    TimeRange(ExactTimeRange),
    AllDay(ExactDate),
}

#[derive(Debug)]
pub struct ExactDuration {
    pub start: ExactDateTime,
    pub duration: u64,
}

#[derive(Debug)]
pub struct Duration {
    pub start: DateTime,
    pub duration: NumVal,
}

#[derive(Debug)]
pub struct TimeRange {
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(Debug)]
pub struct ExactTimeRange {
    pub start: ExactDateTime,
    pub end: ExactDateTime,
}

#[derive(Debug, Copy, Clone)]
pub enum TimeZoneChoice {
    Local,
    Utc,
}

#[derive(Default, Debug, Clone)]
pub struct DateTime {
    pub date: Option<Date>,
    pub time: Option<Time>,
    pub tz: Option<TimeZoneChoice>,
}

#[derive(Debug, Clone)]
pub struct ExactDateTime {
    pub date: ExactDate,
    pub time: ExactTime,
    pub tz: TimeZoneChoice,
}

#[derive(Debug)]
pub enum FlexField {
    NumVal(NumVal),
    NumRange(NumRange),
}

#[derive(Debug)]
pub struct FlexDate {
    pub year: FlexField,
    pub month: FlexField,
    pub day: FlexField,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Date {
    pub year: NumVal,
    pub month: NumVal,
    pub day: NumVal,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExactDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum Tod {
    // Add support for resolving
    AM,
    PM,
}

#[derive(Debug, Copy, Clone)]
pub struct Time {
    pub hour: NumVal,
    pub minute: NumVal,
    pub second: NumVal,
    pub tod: Option<Tod>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExactTime {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

// PERFORMANCE: Figure out how to use &str instead of String
#[derive(Debug)]
pub struct Event {
    pub range: Range,
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct ExactEvent {
    pub range: ExactRange,
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub enum Record {
    Event(Event),
    Occasion(DateTime),
    Note(String),
    FlexOccasion(FlexOccasion),
    FlexEvents(FlexEvents),
}

#[derive(Debug)]
pub enum ExactRecord {
    Event(ExactEvent),
    Note(String),
}

#[derive(Debug)]
pub enum FlexOccasion {
    Filter(Box<dyn Filter<Date>>),
}

#[derive(Debug)]
pub struct FlexEvents {
    pub occasion: FlexOccasion,
    pub events: Vec<Event>,
}

impl DateTime {
    pub fn from_exact(exact: ExactDateTime) -> Self {
        DateTime {
            date: Some(Date {
                year: NumVal::Number(exact.date.year as i64),
                month: NumVal::Number(exact.date.month as i64),
                day: NumVal::Number(exact.date.day as i64),
            }),
            time: Some(Time {
                hour: NumVal::Number(exact.time.hour as i64),
                minute: NumVal::Number(exact.time.minute as i64),
                second: NumVal::Number(exact.time.second as i64),
                tod: None,
            }),
            tz: Some(exact.tz),
        }
    }

    pub fn from_ymd_hms(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        DateTime {
            date: Some(Date {
                year: NumVal::Number(year as i64),
                month: NumVal::Number(month as i64),
                day: NumVal::Number(day as i64),
            }),
            time: Some(Time {
                hour: NumVal::Number(hour as i64),
                minute: NumVal::Number(minute as i64),
                second: NumVal::Number(second as i64),
                tod: None,
            }),
            tz: Some(TimeZoneChoice::Local),
        }
    }
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        DateTime {
            date: Some(Date {
                year: NumVal::Number(year as i64),
                month: NumVal::Number(month as i64),
                day: NumVal::Number(day as i64),
            }),
            time: None,
            tz: Some(TimeZoneChoice::Local),
        }
    }
}

impl Date {
    pub fn new() -> Self {
        use NumVal::Unsure;
        Date {
            year: Unsure,
            month: Unsure,
            day: Unsure,
        }
    }

    pub fn from_naive(naive: NaiveDate) -> Self {
        use NumVal::Number;
        Date {
            year: Number((naive.year()) as i64),
            month: Number((naive.month()) as i64),
            day: Number((naive.day()) as i64),
        }
    }

    pub fn to_naive(&self) -> Option<NaiveDate> {
        use NumVal::*;
        match (self.year, self.month, self.day) {
            (Number(y), Number(m), Number(d)) => Some(NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)?),
            _ => None,
        }
    }
}

impl Time {
    pub fn new() -> Self {
        Time {
            hour: NumVal::Unsure,
            minute: NumVal::Unsure,
            second: NumVal::Unsure,
            tod: None,
        }
    }
}
